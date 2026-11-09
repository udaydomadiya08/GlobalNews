use crate::config::CoreConfig;
use crate::error::CoreError;
use globalnews_platform::{DesktopAdapter, PlatformAdapter};
use globalnews_storage::{DatabaseConfig, DatabaseManager};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct EngineState {
    pub config: CoreConfig,
    pub db: Arc<DatabaseManager>,
    pub platform: Arc<dyn PlatformAdapter>,
}

impl EngineState {
    pub fn new_desktop() -> Result<Self, CoreError> {
        let platform = Arc::new(DesktopAdapter::default());
        let config = CoreConfig::default();
        let db_path = platform.data_dir().join(&config.database_name);

        info!("Initializing GlobalNews Engine on {}", platform.name());
        info!("Database path: {:?}", db_path);

        let db = Arc::new(DatabaseManager::new(DatabaseConfig {
            path: Some(db_path),
            busy_timeout_ms: 5000,
        })?);

        Ok(Self { config, db, platform })
    }

    pub fn new_in_memory() -> Result<Self, CoreError> {
        let platform = Arc::new(DesktopAdapter::default());
        let config = CoreConfig::default();
        let db = Arc::new(DatabaseManager::new_in_memory()?);

        Ok(Self { config, db, platform })
    }

    pub fn new_with_adapter(
        platform: Arc<dyn PlatformAdapter>,
        config: CoreConfig,
    ) -> Result<Self, CoreError> {
        let db_path = platform.data_dir().join(&config.database_name);
        let db = Arc::new(DatabaseManager::new(DatabaseConfig {
            path: Some(db_path),
            busy_timeout_ms: 5000,
        })?);

        Ok(Self { config, db, platform })
    }
}
