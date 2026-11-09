use super::adapter::RawFeedItem;
use super::error::IngestionError;
use chrono::{DateTime, Utc};
use globalnews_contracts::{
    ContentStorageClassification, FeedEndpoint, NormalizedArticle, Source,
};
use regex::Regex;
use std::sync::OnceLock;
use url::Url;
use uuid::Uuid;

static TRACKING_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "fbclid",
    "gclid",
    "ref",
    "source",
    "mc_cid",
    "mc_eid",
    "_hsenc",
    "_hsmi",
];

static HTML_TAG_REGEX: OnceLock<Regex> = OnceLock::new();
static MULTI_SPACE_REGEX: OnceLock<Regex> = OnceLock::new();

fn html_tag_regex() -> &'static Regex {
    HTML_TAG_REGEX.get_or_init(|| Regex::new(r"<[^>]*>").unwrap())
}

fn multi_space_regex() -> &'static Regex {
    MULTI_SPACE_REGEX.get_or_init(|| Regex::new(r"\s+").unwrap())
}

pub struct ArticleNormalizer;

impl ArticleNormalizer {
    /// Canonicalizes a raw article link by stripping tracking tokens, fragments, and redundant components.
    pub fn canonicalize_url(raw_url: &str, base_url: Option<&str>) -> Result<String, IngestionError> {
        let trimmed = raw_url.trim();
        if trimmed.is_empty() {
            return Err(IngestionError::Normalization("URL cannot be empty".into()));
        }

        let mut parsed = if let Some(base) = base_url {
            let base_parsed = Url::parse(base)
                .map_err(|e| IngestionError::Normalization(format!("Invalid base URL '{}': {}", base, e)))?;
            base_parsed
                .join(trimmed)
                .map_err(|e| IngestionError::Normalization(format!("Failed to resolve relative URL '{}': {}", trimmed, e)))?
        } else {
            Url::parse(trimmed)
                .map_err(|e| IngestionError::Normalization(format!("Invalid URL '{}': {}", trimmed, e)))?
        };

        // Remove fragment (#anchor)
        parsed.set_fragment(None);

        // Filter tracking query parameters
        let filtered_pairs: Vec<(String, String)> = parsed
            .query_pairs()
            .filter(|(key, _)| !TRACKING_PARAMS.contains(&key.to_ascii_lowercase().as_str()))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        if filtered_pairs.is_empty() {
            parsed.set_query(None);
        } else {
            let mut serializer = url::form_urlencoded::Serializer::new(String::new());
            for (k, v) in filtered_pairs {
                serializer.append_pair(&k, &v);
            }
            parsed.set_query(Some(&serializer.finish()));
        }

        // Clean trailing slash on paths
        let mut canon = parsed.to_string();
        if canon.ends_with('/') && !canon.ends_with("://") && parsed.path() != "/" {
            canon.pop();
        }

        Ok(canon)
    }

    /// Strips HTML tags and unescapes entities safely.
    pub fn sanitize_text(text: &str, max_len: Option<usize>) -> String {
        let no_tags = html_tag_regex().replace_all(text, " ");
        let unescaped = no_tags
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&apos;", "'");

        let normalized = multi_space_regex().replace_all(&unescaped, " ").trim().to_string();

        if let Some(limit) = max_len {
            if normalized.len() > limit {
                let mut truncated = normalized[..limit].to_string();
                if let Some(last_space) = truncated.rfind(' ') {
                    truncated.truncate(last_space);
                }
                truncated.push_str("...");
                return truncated;
            }
        }

        normalized
    }

    /// Parses various date formats into UTC ISO-8601 string while preserving the raw source string.
    pub fn parse_date(raw_date: Option<&str>) -> (Option<String>, Option<String>) {
        let raw = match raw_date {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => return (None, None),
        };

        // 1. Try RFC 2822 (Standard RSS 2.0 pubDate)
        if let Ok(dt) = DateTime::parse_from_rfc2822(&raw) {
            let utc: DateTime<Utc> = dt.into();
            return (Some(utc.to_rfc3339()), Some(raw));
        }

        // 2. Try RFC 3339 / ISO 8601 (Standard Atom)
        if let Ok(dt) = DateTime::parse_from_rfc3339(&raw) {
            let utc: DateTime<Utc> = dt.into();
            return (Some(utc.to_rfc3339()), Some(raw));
        }

        // 3. Fallback: Return raw string honestly with None normalized
        (None, Some(raw))
    }

    /// Transforms a RawFeedItem into a fully validated NormalizedArticle.
    pub fn normalize(
        item: &RawFeedItem,
        source: &Source,
        endpoint: Option<&FeedEndpoint>,
    ) -> Result<NormalizedArticle, IngestionError> {
        let base = endpoint.map(|e| e.endpoint_url.as_str()).unwrap_or(source.feed_url.as_str());
        let canonical_url = Self::canonicalize_url(&item.link, Some(base))?;
        let clean_title = Self::sanitize_text(&item.title, Some(300));

        if clean_title.is_empty() {
            return Err(IngestionError::Normalization("Article title cannot be empty".into()));
        }

        let excerpt = item.description.as_deref().or(item.content.as_deref()).map(|d| {
            Self::sanitize_text(d, Some(500))
        });

        let (published_at, raw_published_at) = Self::parse_date(item.pub_date_raw.as_deref());
        let now_utc = Utc::now().to_rfc3339();

        let classification = if excerpt.is_some() {
            ContentStorageClassification::PermittedExcerpt
        } else {
            ContentStorageClassification::MetadataOnly
        };

        let ep_id = endpoint.map(|e| e.id.clone());
        let provenance = format!(
            "Ingested from {} ({}) via endpoint {}",
            source.name,
            source.id,
            endpoint.map(|e| e.endpoint_url.as_str()).unwrap_or(&source.feed_url)
        );

        let id = Uuid::new_v4().to_string();
        let canon_hash = NormalizedArticle::compute_hash(&canonical_url);
        let td_hash = NormalizedArticle::compute_title_date_hash(&clean_title, published_at.as_deref());

        let mut article = NormalizedArticle {
            id,
            source_id: source.id.clone(),
            endpoint_id: ep_id,
            canonical_url,
            original_url: item.link.clone(),
            title: clean_title,
            description_excerpt: excerpt,
            content_classification: classification,
            published_at,
            raw_published_at,
            retrieved_at: now_utc,
            updated_at: None,
            language: source.supported_languages.first().cloned(),
            authors: item.authors.clone(),
            feed_categories: item.categories.clone(),
            source_provenance: provenance,
            is_syndicated: false,
            canonical_url_hash: canon_hash,
            title_date_hash: td_hash,
        };

        article.validate()?;
        Ok(article)
    }
}
