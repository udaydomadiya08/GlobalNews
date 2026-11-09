use globalnews_contracts::ContractError;
use globalnews_storage::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("HTTP request error: {0}")]
    Http(String),

    #[error("Network timeout during feed retrieval: {0}")]
    Timeout(String),

    #[error("Rate limited by source (HTTP 429), retry after {retry_after_secs:?} seconds")]
    RateLimited { retry_after_secs: Option<u64> },

    #[error("HTTP status {status}: {message}")]
    HttpStatus { status: u16, message: String },

    #[error("Payload exceeded size limit ({size_bytes} bytes > {max_bytes} bytes)")]
    PayloadTooLarge { size_bytes: usize, max_bytes: usize },

    #[error("Unexpected MIME type: expected {expected}, got {actual}")]
    MimeTypeMismatch { expected: String, actual: String },

    #[error("XML parsing error: {0}")]
    XmlParse(String),

    #[error("JSON parsing error: {0}")]
    JsonParse(String),

    #[error("SSRF defense blocked request to forbidden target: {0}")]
    SsrfBlocked(String),

    #[error("Article normalization error: {0}")]
    Normalization(String),

    #[error("Database storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Contract validation error: {0}")]
    Contract(#[from] ContractError),

    #[error("Source is currently disabled")]
    SourceDisabled,

    #[error("Source is in exponential backoff until {0}")]
    InBackoff(String),
}

