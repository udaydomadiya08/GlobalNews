use crate::error::ContractError;
use serde::{Deserialize, Serialize};

/// High-level status of an endpoint ingestion execution run.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestionStatus {
    Running,
    Success,
    NotModified,
    PartialError,
    Failed,
}

impl IngestionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::NotModified => "not_modified",
            Self::PartialError => "partial_error",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "running" => Some(Self::Running),
            "success" => Some(Self::Success),
            "not_modified" => Some(Self::NotModified),
            "partial_error" => Some(Self::PartialError),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Audit record for a specific execution run of an ingestion endpoint.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct IngestionRun {
    pub id: String,
    pub endpoint_id: String,
    pub started_at: String, // RFC 3339 UTC
    pub completed_at: Option<String>,
    pub status: IngestionStatus,
    pub http_status_code: Option<u16>,
    pub articles_fetched: usize,
    pub articles_new: usize,
    pub articles_duplicate: usize,
    pub error_message: Option<String>,
}

impl IngestionRun {
    pub fn new(id: impl Into<String>, endpoint_id: impl Into<String>, started_at: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            endpoint_id: endpoint_id.into(),
            started_at: started_at.into(),
            completed_at: None,
            status: IngestionStatus::Running,
            http_status_code: None,
            articles_fetched: 0,
            articles_new: 0,
            articles_duplicate: 0,
            error_message: None,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "id",
                message: "IngestionRun ID cannot be empty".into(),
            });
        }
        if self.endpoint_id.trim().is_empty() {
            return Err(ContractError::ValidationError {
                field: "endpoint_id",
                message: "Endpoint ID cannot be empty".into(),
            });
        }
        Ok(())
    }
}

/// Category of ingestion or retrieval error.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    NetworkTimeout,
    TlsError,
    Http429,
    Http4xx,
    Http5xx,
    PayloadTooLarge,
    MimeTypeMismatch,
    XmlParseError,
    JsonParseError,
    SsrfBlocked,
    Other,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NetworkTimeout => "network_timeout",
            Self::TlsError => "tls_error",
            Self::Http429 => "http_429",
            Self::Http4xx => "http_4xx",
            Self::Http5xx => "http_5xx",
            Self::PayloadTooLarge => "payload_too_large",
            Self::MimeTypeMismatch => "mime_type_mismatch",
            Self::XmlParseError => "xml_parse_error",
            Self::JsonParseError => "json_parse_error",
            Self::SsrfBlocked => "ssrf_blocked",
            Self::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "network_timeout" => Self::NetworkTimeout,
            "tls_error" => Self::TlsError,
            "http_429" => Self::Http429,
            "http_4xx" => Self::Http4xx,
            "http_5xx" => Self::Http5xx,
            "payload_too_large" => Self::PayloadTooLarge,
            "mime_type_mismatch" => Self::MimeTypeMismatch,
            "xml_parse_error" => Self::XmlParseError,
            "json_parse_error" => Self::JsonParseError,
            "ssrf_blocked" => Self::SsrfBlocked,
            _ => Self::Other,
        }
    }
}

/// Recorded retrieval failure for health tracking and debugging.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RetrievalError {
    pub id: String,
    pub run_id: Option<String>,
    pub endpoint_id: String,
    pub error_kind: ErrorKind,
    pub details: String,
    pub occurred_at: String, // RFC 3339 UTC
}

/// Result returned from deduplicating insert operation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum InsertResult {
    Inserted(String), // Returns ID of newly inserted article
    Duplicate {
        existing_id: String,
        reason: String,
    },
}
