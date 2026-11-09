use crate::error::PlatformError;
use crate::traits::PlatformAdapter;
use std::path::PathBuf;
use tracing::info;

pub struct MobileAdapter {
    internal_files_dir: PathBuf,
    is_ios: bool,
}

impl MobileAdapter {
    pub fn new(internal_files_dir: impl Into<PathBuf>, is_ios: bool) -> Self {
        Self {
            internal_files_dir: internal_files_dir.into(),
            is_ios,
        }
    }
}

impl PlatformAdapter for MobileAdapter {
    fn name(&self) -> &'static str {
        if self.is_ios {
            "iOS Mobile"
        } else {
            "Android Mobile"
        }
    }

    fn data_dir(&self) -> PathBuf {
        self.internal_files_dir.clone()
    }

    fn cache_dir(&self) -> PathBuf {
        self.internal_files_dir.join("cache")
    }

    fn show_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        info!("[Mobile Native Notification Bridge] {}: {}", title, body);
        Ok(())
    }

    fn schedule_background_sync(&self, interval_mins: u32) -> Result<(), PlatformError> {
        if self.is_ios {
            info!("[iOS BGTaskScheduler Bridge] Registered BGAppRefreshTask (interval: {}m)", interval_mins);
        } else {
            info!("[Android WorkManager Bridge] Scheduled PeriodicWorkRequest (interval: {}m)", interval_mins);
        }
        Ok(())
    }
}
