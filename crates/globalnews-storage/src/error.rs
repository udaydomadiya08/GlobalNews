use globalnews_contracts::ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite database error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("Contract validation error: {0}")]
    ContractError(#[from] ContractError),

    #[error("Migration error at version {version}: {message}")]
    MigrationError { version: i32, message: String },

    #[error("Record not found: {entity} with id '{id}'")]
    NotFound { entity: &'static str, id: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lock contention or pool acquisition timeout")]
    LockTimeout,
}
