use globalnews_contracts::ContractError;
use globalnews_platform::PlatformError;
use globalnews_storage::StorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Contract validation error: {0}")]
    Contract(#[from] ContractError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task cancelled or engine shut down")]
    Cancelled,
}
