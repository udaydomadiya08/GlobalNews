use crate::error::ContractError;
use serde::{Deserialize, Serialize};

/// Canonical category of a news, sensor, or intelligence source.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceCategory {
    Geopolitical,
    DisasterSeismic,
    ScienceTech,
    EconomicMarkets,
    HumanitarianHealth,
    EnvironmentalClimate,
    GeneralNews,
}

impl SourceCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Geopolitical => "geopolitical",
            Self::DisasterSeismic => "disaster_seismic",
            Self::ScienceTech => "science_tech",
            Self::EconomicMarkets => "economic_markets",
            Self::HumanitarianHealth => "humanitarian_health",
            Self::EnvironmentalClimate => "environmental_climate",
            Self::GeneralNews => "general_news",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "geopolitical" => Self::Geopolitical,
            "disaster_seismic" => Self::DisasterSeismic,
            "science_tech" => Self::ScienceTech,
            "economic_markets" => Self::EconomicMarkets,
            "humanitarian_health" => Self::HumanitarianHealth,
            "environmental_climate" => Self::EnvironmentalClimate,
            _ => Self::GeneralNews,
        }
    }
}

/// Supported format of a feed or endpoint payload.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeedFormat {
    Rss2,
    Atom,
    JsonStructured,
}

impl FeedFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rss2 => "rss2",
            Self::Atom => "atom",
            Self::JsonStructured => "json_structured",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "rss2" | "rss" => Some(Self::Rss2),
            "atom" => Some(Self::Atom),
            "json_structured" | "geojson" | "json" => Some(Self::JsonStructured),
            _ => None,
        }
    }
}

/// HTTP retrieval method for feed endpoints.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalMethod {
    HttpGet,
    HttpConditionalGet,
}

impl RetrievalMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HttpGet => "http_get",
            Self::HttpConditionalGet => "http_conditional_get",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "http_conditional_get" => Self::HttpConditionalGet,
            _ => Self::HttpGet,
        }
    }
}

/// An individual feed or API endpoint associated with a publisher source.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeedEndpoint {
    pub id: String,
    pub source_id: String,
    pub endpoint_url: String,
    pub feed_format: FeedFormat,
    pub retrieval_method: RetrievalMethod,
    pub polling_interval_secs: u32,
    pub rate_limit_per_minute: Option<u32>,
    pub enabled: bool,
    pub priority: u32,
    pub created_at: String, // RFC 3339 UTC
    pub updated_at: String, // RFC 3339 UTC
}

impl FeedEndpoint {
    pub fn new(
        id: impl Into<String>,
        source_id: impl Into<String>,
        endpoint_url: impl Into<String>,
        feed_format: FeedFormat,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: id.into(),
            source_id: source_id.into(),
            endpoint_url: endpoint_url.into(),
            feed_format,
            retrieval_method: RetrievalMethod::HttpConditionalGet,
            polling_interval_secs: 1800,
            rate_limit_per_minute: None,
            enabled: true,
            priority: 10,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "FeedEndpoint ID cannot be empty".into(),
            });
        }
        if self.source_id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "source_id",
                message: "Source ID cannot be empty".into(),
            });
        }
        if self.endpoint_url.trim().is_empty() || !self.endpoint_url.starts_with("http") {
            return Err(ContractError::ValidationError {
                field: "endpoint_url",
                message: "Endpoint URL must be an HTTP/HTTPS URL".into(),
            });
        }
        if self.polling_interval_secs < 60 {
            return Err(ContractError::ValidationError {
                field: "polling_interval_secs",
                message: "Polling interval cannot be lower than 60 seconds".into(),
            });
        }
        Ok(())
    }
}

/// HTTP caching and progress checkpoint state for a feed endpoint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct FeedCheckpoint {
    pub endpoint_id: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_retrieval_at: Option<String>, // RFC 3339 UTC
    pub last_status_code: Option<u16>,
    pub last_successful_article_count: u32,
    pub content_hash: Option<String>,
}

/// Operational health and backoff state for an intelligence source.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SourceHealth {
    pub source_id: String,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub rate_limited_count: u64,
    pub consecutive_failures: u32,
    pub backoff_until: Option<String>, // RFC 3339 UTC
    pub last_error_message: Option<String>,
    pub last_error_at: Option<String>, // RFC 3339 UTC
    pub reliability_score: f64,
    pub updated_at: String, // RFC 3339 UTC
}

impl SourceHealth {
    pub fn new(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rate_limited_count: 0,
            consecutive_failures: 0,
            backoff_until: None,
            last_error_message: None,
            last_error_at: None,
            reliability_score: 1.0,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Represents an authorized feed publisher, sensor network, or intelligence agency.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Source {
    pub id: String,             // E.g. "usgs-earthquakes", "bbc-world"
    pub name: String,           // Publisher display name
    pub tier: u8,               // Tier 1 (Sensor) to Tier 5 (General)
    pub feed_url: String,       // Primary / legacy endpoint URL
    pub feed_type: String,      // "geojson", "rss2", "atom", "reliefweb_api"
    pub reliability_score: f64, // Base reputation weight (0.1 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>, // HTTP caching header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>, // HTTP caching header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_polled_at: Option<String>, // RFC 3339 UTC

    // Phase 1.2 Registry Metadata Extensions
    #[serde(default)]
    pub canonical_domain: String, // E.g. "earthquake.usgs.gov", "bbc.co.uk"
    #[serde(default = "default_category")]
    pub category: SourceCategory,
    #[serde(default)]
    pub supported_languages: Vec<String>, // E.g. ["en"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geographic_coverage: Option<String>, // E.g. "GLOBAL", "US"
    #[serde(default)]
    pub attribution_requirements: String, // E.g. "USGS Open Data public domain"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licence_policy_url: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_category() -> SourceCategory {
    SourceCategory::GeneralNews
}

fn default_true() -> bool {
    true
}

impl Source {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        tier: u8,
        feed_url: impl Into<String>,
        feed_type: impl Into<String>,
        reliability_score: f64,
    ) -> Self {
        let f_url = feed_url.into();
        let domain = url::Url::parse(&f_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default();

        Self {
            id: id.into(),
            name: name.into(),
            tier,
            feed_url: f_url,
            feed_type: feed_type.into(),
            reliability_score,
            etag: None,
            last_modified: None,
            last_polled_at: None,
            canonical_domain: domain,
            category: SourceCategory::GeneralNews,
            supported_languages: vec!["en".to_string()],
            geographic_coverage: Some("GLOBAL".to_string()),
            attribution_requirements: String::new(),
            licence_policy_url: None,
            enabled: true,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "Source ID cannot be empty".into(),
            });
        }
        if self.feed_url.trim().is_empty() || !self.feed_url.starts_with("http") {
            return Err(ContractError::ValidationError {
                field: "feed_url",
                message: "Feed URL must be an HTTP/HTTPS URL".into(),
            });
        }
        if !(1..=5).contains(&self.tier) {
            return Err(ContractError::ValidationError {
                field: "tier",
                message: "Source tier must be between 1 and 5".into(),
            });
        }
        if !(0.1..=1.0).contains(&self.reliability_score) {
            return Err(ContractError::InvalidScoreRange(
                self.reliability_score,
                0.1,
                1.0,
            ));
        }
        Ok(())
    }
}
