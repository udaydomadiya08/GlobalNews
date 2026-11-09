use globalnews_contracts::{MacroEvent, MicroEvent};
use globalnews_core::EngineState;
use globalnews_storage::Repository;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid request parameters: {0}")]
    InvalidRequest(String),

    #[error("Internal engine error: {0}")]
    Internal(String),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HealthResponse {
    pub status: &'static str,
    pub engine_version: &'static str,
    pub schema_version: &'static str,
    pub platform: String,
    pub database_active: bool,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EventFilter {
    pub min_importance: Option<f64>,
    pub category_code: Option<String>,
    pub bounding_box: Option<[f64; 4]>, // [min_lat, min_lon, max_lat, max_lon]
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub event_id: String,
    pub title: String,
    pub rank: f64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageStats {
    pub current_version: i32,
    pub quota_mb: u64,
    pub estimated_usage_mb: f64,
}

// -----------------------------------------------------------------------------
// Command Handlers (Invoked directly via Tauri In-Process IPC)
// -----------------------------------------------------------------------------

pub fn handle_get_health(state: &EngineState) -> HealthResponse {
    HealthResponse {
        status: "OK",
        engine_version: "0.1.0",
        schema_version: globalnews_contracts::CURRENT_SCHEMA_VERSION,
        platform: state.platform.name().to_string(),
        database_active: true,
    }
}

pub fn handle_search_events(
    state: &EngineState,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<SearchResult>, CommandError> {
    if query.trim().is_empty() {
        return Err(CommandError::InvalidRequest(
            "Search query cannot be empty".into(),
        ));
    }
    let lim = limit.unwrap_or(20);
    state
        .db
        .with_reader(|conn| {
            let rows = Repository::search_events(conn, &query, lim)?;

            let results = rows
                .into_iter()
                .map(|(event_id, title, rank)| SearchResult {
                    event_id,
                    title,
                    rank,
                })
                .collect();

            Ok(results)
        })
        .map_err(|e| CommandError::Database(e.to_string()))
}

pub fn handle_get_micro_event(
    state: &EngineState,
    id: String,
) -> Result<Option<MicroEvent>, CommandError> {
    state
        .db
        .with_reader(|conn| Repository::get_micro_event(conn, &id))
        .map_err(|e| CommandError::Database(e.to_string()))
}

pub fn handle_get_macro_event(
    state: &EngineState,
    id: String,
) -> Result<Option<MacroEvent>, CommandError> {
    state
        .db
        .with_reader(|conn| Repository::get_macro_event(conn, &id))
        .map_err(|e| CommandError::Database(e.to_string()))
}
