use crate::error::ContractError;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Represents an individual ingested news document or sensor record.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Article {
    pub id: String, // UUID v4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub micro_event_id: Option<String>, // Associated micro-event (nullable)
    pub url: String, // Canonical publisher URL
    pub title: String, // Cleaned title
    pub body_excerpt: String, // 2-3 sentence factual TextRank extract
    pub language: String, // ISO 639-1 code (e.g. "en", "ja", "ar")
    pub source_id: String, // Foreign key to Source
    pub content_hash: String, // SHA-256 hash of normalized text
    pub published_at: String, // RFC 3339 UTC
    pub ingested_at: String, // RFC 3339 UTC
    pub is_syndicated: bool, // True if identified as wire duplicate
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
