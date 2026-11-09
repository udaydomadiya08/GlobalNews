use crate::error::PlatformError;
use std::path::PathBuf;

pub trait PlatformAdapter: Send + Sync {
    /// Human-readable platform name (e.g. "macOS", "Windows", "Linux", "Android", "iOS")
    fn name(&self) -> &'static str;

    /// Root directory for persistent database storage
    fn data_dir(&self) -> PathBuf;

    /// Root directory for temporary cache files
    fn cache_dir(&self) -> PathBuf;

    /// Checks if device is currently on a metered network
    fn is_metered_network(&self) -> bool {
        false
    }

    /// Checks if device is currently running on battery
    fn is_on_battery(&self) -> bool {
        false
    }

    /// Dispatches a native OS desktop or mobile notification
    fn show_notification(&self, title: &str, body: &str) -> Result<(), PlatformError>;

    /// Schedules a native periodic background task
    fn schedule_background_sync(&self, interval_mins: u32) -> Result<(), PlatformError>;
}
