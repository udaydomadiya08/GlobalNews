use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ContractError {
    #[error("Validation error in field '{field}': {message}")]
    ValidationError {
        field: &'static str,
        message: String,
    },

    #[error("Invalid latitude: {0} (must be between -90.0 and 90.0)")]
    InvalidLatitude(f64),

    #[error("Invalid longitude: {0} (must be between -180.0 and 180.0)")]
    InvalidLongitude(f64),

    #[error("Invalid score: {0} (must be between {1} and {2})")]
    InvalidScoreRange(f64, f64, f64),

    #[error("Invalid ISO country code: '{0}' (must be 2 uppercase ASCII letters)")]
    InvalidCountryCode(String),

    #[error("Invalid timestamp format: '{0}' (must be RFC 3339 / ISO 8601 UTC)")]
    InvalidTimestamp(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
