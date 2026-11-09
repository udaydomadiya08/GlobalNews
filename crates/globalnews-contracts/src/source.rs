use crate::error::ContractError;
use serde::{Deserialize, Serialize};

/// Represents an authorized feed publisher, sensor network, or agency.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Source {
    pub id: String,             // E.g. "usgs-earthquakes", "reuters-world"
    pub name: String,           // Display name
    pub tier: u8,               // Tier 1 (Sensor) to Tier 5 (Science)
    pub feed_url: String,       // Endpoint URL
    pub feed_type: String,      // "geojson", "rss2", "atom", "reliefweb_api"
    pub reliability_score: f64, // Base reputation weight (0.1 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>, // HTTP caching header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>, // HTTP caching header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_polled_at: Option<String>, // RFC 3339 UTC
}

impl Source {
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
