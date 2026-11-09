use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Platform capability not supported: {0}")]
    Unsupported(String),

    #[error("Platform IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Notification failure: {0}")]
    NotificationError(String),

    #[error("Native scheduler error: {0}")]
    SchedulerError(String),
}
