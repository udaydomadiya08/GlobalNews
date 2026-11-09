use crate::error::ContractError;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Content rights and retention classification for an ingested article.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentStorageClassification {
    MetadataOnly,
    PermittedExcerpt,
    FullContentPermitted,
}

impl ContentStorageClassification {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MetadataOnly => "metadata_only",
            Self::PermittedExcerpt => "permitted_excerpt",
            Self::FullContentPermitted => "full_content_permitted",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "permitted_excerpt" => Self::PermittedExcerpt,
            "full_content_permitted" => Self::FullContentPermitted,
            _ => Self::MetadataOnly,
        }
    }
}

/// Deduplication fingerprint for tracking duplicates and wire syndication.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ArticleFingerprint {
    pub fingerprint_hash: String,
    pub hash_type: String, // "canonical_url", "title_date", "guid"
    pub article_id: String,
    pub source_id: String,
    pub first_seen_at: String, // RFC 3339 UTC
}

/// Normalized canonical article representation produced by ingestion pipeline.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NormalizedArticle {
    pub id: String, // UUID v4
    pub source_id: String, // Foreign key to Source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_id: Option<String>, // Associated feed endpoint
    pub canonical_url: String, // Normalized canonical URL
    pub original_url: String, // Raw unmodified link from source
    pub title: String, // Sanitized, unescaped title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_excerpt: Option<String>, // Bounded excerpt
    pub content_classification: ContentStorageClassification,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>, // Normalized ISO-8601 UTC string (nullable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_published_at: Option<String>, // Raw unparsed string from source feed
    pub retrieved_at: String, // RFC 3339 UTC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>, // RFC 3339 UTC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>, // ISO 639-1 code if known
    pub authors: Vec<String>,
    pub feed_categories: Vec<String>,
    pub source_provenance: String, // Attribution and provenance statement
    pub is_syndicated: bool,
    pub canonical_url_hash: String, // SHA-256 of canonical URL
    pub title_date_hash: String, // SHA-256 of normalized title + pub date day
}

impl NormalizedArticle {
    pub fn new(
        id: impl Into<String>,
        source_id: impl Into<String>,
        canonical_url: impl Into<String>,
        original_url: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        let c_url = canonical_url.into();
        let orig_url = original_url.into();
        let t = title.into();
        let now = chrono::Utc::now().to_rfc3339();

        let c_hash = Self::compute_hash(&c_url);
        let td_hash = Self::compute_title_date_hash(&t, None);

        Self {
            id: id.into(),
            source_id: source_id.into(),
            endpoint_id: None,
            canonical_url: c_url,
            original_url: orig_url,
            title: t,
            description_excerpt: None,
            content_classification: ContentStorageClassification::MetadataOnly,
            published_at: None,
            raw_published_at: None,
            retrieved_at: now,
            updated_at: None,
            language: None,
            authors: Vec::new(),
            feed_categories: Vec::new(),
            source_provenance: String::new(),
            is_syndicated: false,
            canonical_url_hash: c_hash,
            title_date_hash: td_hash,
        }
    }

    pub fn compute_hash(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn compute_title_date_hash(title: &str, published_at: Option<&str>) -> String {
        let clean_title = title.trim().to_lowercase();
        let date_part = match published_at {
            Some(dt) if dt.len() >= 10 => &dt[0..10], // Extract YYYY-MM-DD
            _ => "unknown-date",
        };
        let combined = format!("{}_{}", clean_title, date_part);
        Self::compute_hash(&combined)
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "NormalizedArticle ID cannot be empty".into(),
            });
        }
        if self.source_id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "source_id",
                message: "Source ID cannot be empty".into(),
            });
        }
        if self.canonical_url.trim().is_empty() || !self.canonical_url.starts_with("http") {
            return Err(ContractError::ValidationError {
                field: "canonical_url",
                message: "Canonical URL must be an HTTP/HTTPS URL".into(),
            });
        }
        if self.title.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "title",
                message: "Article title cannot be empty".into(),
            });
        }
        if let Some(ref pub_at) = self.published_at {
            DateTime::parse_from_rfc3339(pub_at)
                .map_err(|_| ContractError::InvalidTimestamp(pub_at.clone()))?;
        }
        DateTime::parse_from_rfc3339(&self.retrieved_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.retrieved_at.clone()))?;
        Ok(())
    }
}

/// Legacy / compat Article struct preserving Phase 1.1 contracts.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Article {
    pub id: String, // UUID v4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro_event_id: Option<String>,
    pub url: String,
    pub title: String,
    pub body_excerpt: String,
    pub language: String,
    pub source_id: String,
    pub content_hash: String,
    pub published_at: String,
    pub ingested_at: String,
    pub is_syndicated: bool,
}

impl Article {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "Article ID cannot be empty".into(),
            });
        }
        if self.url.trim().is_empty() || !self.url.starts_with("http") {
            return Err(ContractError::ValidationError {
                field: "url",
                message: "Article URL must be an HTTP/HTTPS URL".into(),
            });
        }
        if self.title.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "title",
                message: "Article title cannot be empty".into(),
            });
        }
        DateTime::parse_from_rfc3339(&self.published_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.published_at.clone()))?;
        DateTime::parse_from_rfc3339(&self.ingested_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.ingested_at.clone()))?;
        Ok(())
    }
}
