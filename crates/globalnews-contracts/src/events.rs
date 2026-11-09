use crate::error::ContractError;
use crate::location::GeographicLocation;
use crate::scoring::{ConfidenceScore, ImportanceScore};
use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventStatus {
    Emerging,
    Developing,
    Confirmed,
    Disputed,
    Archive,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MacroEventStatus {
    Active,
    Dormant,
    Concluded,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationType {
    Causes,
    Disrupts,
    Escalates,
    Resolves,
    Precedes,
}

/// Represents a localized incident node plotted as a hotspot on the 3D Earth.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MicroEvent {
    pub id: String,                     // UUID v4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macro_event_id: Option<String>, // Parent container (e.g. war or crisis)
    pub title: String,                  // Canonical event headline
    pub summary: String,                // 3-sentence synthesized extractive dossier
    pub category_code: String,          // Standard IPTC Media Topic (e.g. "03000000")
    pub importance: ImportanceScore,    // Drives hotspot radius
    pub confidence: ConfidenceScore,    // Drives hotspot opacity / cadence
    pub location: GeographicLocation,   // Spherical plotting coordinates
    pub status: EventStatus,            // Lifecycle state
    pub first_seen_at: String,          // RFC 3339 UTC
    pub last_updated_at: String,        // RFC 3339 UTC
    pub citation_count: u32,            // Count of linked citations
}

impl MicroEvent {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "MicroEvent ID cannot be empty".into(),
            });
        }
        if self.title.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "title",
                message: "Event title cannot be empty".into(),
            });
        }
        self.importance.validate()?;
        self.confidence.validate()?;
        self.location.validate()?;
        DateTime::parse_from_rfc3339(&self.first_seen_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.first_seen_at.clone()))?;
        DateTime::parse_from_rfc3339(&self.last_updated_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.last_updated_at.clone()))?;
        Ok(())
    }
}

/// Represents a persistent macro development container (multi-month or multi-year).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MacroEvent {
    pub id: String,                     // UUID v4
    pub title: String,                  // E.g. "Russo-Ukrainian War"
    pub category_code: String,          // IPTC code
    pub summary: String,                // Macro overview
    pub status: MacroEventStatus,       // Active, Dormant, Concluded
    pub bounding_box: [f64; 4],         // [min_lat, min_lon, max_lat, max_lon]
    pub created_at: String,             // RFC 3339 UTC
    pub updated_at: String,             // RFC 3339 UTC
    pub active_micro_event_count: u32,  // Count of child micro-events
}

impl MacroEvent {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "MacroEvent ID cannot be empty".into(),
            });
        }
        if self.title.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "title",
                message: "MacroEvent title cannot be empty".into(),
            });
        }
        DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.created_at.clone()))?;
        DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.updated_at.clone()))?;
        Ok(())
    }
}

/// Represents a directed semantic relationship between two events.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EventRelationship {
    pub source_event_id: String,
    pub target_event_id: String,
    pub relationship_type: RelationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl EventRelationship {
    pub fn validate(&self) -> Result<(), ContractError> {
        if self.source_event_id.trim().is_empty() || self.target_event_id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "event_ids",
                message: "Source and target event IDs cannot be empty".into(),
            });
        }
        if self.source_event_id == self.target_event_id {
            return Err(ContractError::ValidationError {
                field: "target_event_id",
                message: "Event cannot have a relationship with itself".into(),
            });
        }
        Ok(())
    }
}
