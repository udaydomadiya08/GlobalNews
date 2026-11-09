use crate::error::PlatformError;
use crate::traits::PlatformAdapter;
use std::path::PathBuf;
use tracing::info;

pub struct DesktopAdapter {
    app_name: &'static str,
}

impl Default for DesktopAdapter {
    fn default() -> Self {
        Self {
            app_name: "globalnews",
        }
    }
}

impl PlatformAdapter for DesktopAdapter {
    fn name(&self) -> &'static str {
        #[cfg(target_os = "macos")]
        return "macOS Desktop";
        #[cfg(target_os = "windows")]
        return "Windows Desktop";
        #[cfg(target_os = "linux")]
        return "Linux Desktop";
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return "Generic Desktop";
    }

    fn data_dir(&self) -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".into());

        #[cfg(target_os = "macos")]
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join(self.app_name);

        #[cfg(target_os = "windows")]
        return PathBuf::from(std::env::var("LOCALAPPDATA").unwrap_or(home))
            .join(self.app_name);

        #[cfg(target_os = "linux")]
        return PathBuf::from(std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
            format!("{}/.local/share", home)
        }))
        .join(self.app_name);

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return PathBuf::from(home).join(format!(".{}", self.app_name));
    }

    fn cache_dir(&self) -> PathBuf {
        self.data_dir().join("cache")
    }

    fn show_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        info!("[Desktop Notification] {}: {}", title, body);
        Ok(())
    }

    fn schedule_background_sync(&self, interval_mins: u32) -> Result<(), PlatformError> {
        info!("[Desktop Background Scheduler] Sync interval configured to {} mins", interval_mins);
        Ok(())
    }
}
