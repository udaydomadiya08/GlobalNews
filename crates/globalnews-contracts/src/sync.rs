use crate::error::ContractError;
use crate::events::{EventRelationship, MacroEvent, MicroEvent};
use chrono::DateTime;
use serde::{Deserialize, Serialize};

pub const CURRENT_SCHEMA_VERSION: &str = "1.0.0";

/// Cross-platform synchronization envelope for peer or client state hydration.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SyncEnvelope {
    pub schema_version: String, // E.g. "1.0.0"
    pub client_id: String,      // Ephemeral installation or node UUID
    pub generated_at: String,   // RFC 3339 UTC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_cursor: Option<String>,
    pub micro_events: Vec<MicroEvent>,
    pub macro_events: Vec<MacroEvent>,
    pub relationships: Vec<EventRelationship>,
}

impl SyncEnvelope {
    pub fn new(
        client_id: impl Into<String>,
        last_sync_cursor: Option<String>,
        micro_events: Vec<MicroEvent>,
        macro_events: Vec<MacroEvent>,
        relationships: Vec<EventRelationship>,
    ) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            client_id: client_id.into(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            last_sync_cursor,
            micro_events,
            macro_events,
            relationships,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(ContractError::ValidationError {
                field: "schema_version",
                message: format!(
                    "Unsupported schema version: '{}' (expected '{}')",
                    self.schema_version, CURRENT_SCHEMA_VERSION
                ),
            });
        }
        DateTime::parse_from_rfc3339(&self.generated_at)
            .map_err(|_| ContractError::InvalidTimestamp(self.generated_at.clone()))?;
        for ev in &self.micro_events {
            ev.validate()?;
        }
        for ma in &self.macro_events {
            ma.validate()?;
        }
        for rel in &self.relationships {
            rel.validate()?;
        }
        Ok(())
    }
}
