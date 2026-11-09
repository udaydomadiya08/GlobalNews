use crate::error::ContractError;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TimelineEntry {
    pub id: String,
    pub micro_event_id: String,
    pub timestamp: String, // RFC 3339 UTC
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_url: Option<String>,
}

impl TimelineEntry {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() || self.micro_event_id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "Timeline IDs cannot be empty".into(),
            });
        }
        DateTime::parse_from_rfc3339(&self.timestamp)
            .map_err(|_| ContractError::InvalidTimestamp(self.timestamp.clone()))?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EvidenceRecord {
    pub id: String,
    pub micro_event_id: String,
    pub source_id: String,
    pub article_id: String,
    pub corroboration_weight: f64,
}

impl EvidenceRecord {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "EvidenceRecord ID cannot be empty".into(),
            });
        }
        if !(0.0..=1.0).contains(&self.corroboration_weight) {
            return Err(ContractError::InvalidScoreRange(
                self.corroboration_weight,
                0.0,
                1.0,
            ));
        }
        Ok(())
    }
}
