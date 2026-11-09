use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CoreConfig {
    pub storage_quota_mb: u64,
    pub poll_interval_ac_mins: u32,
    pub poll_interval_battery_mins: u32,
    pub enable_realtime_hazard_alerts: bool,
    pub database_name: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            storage_quota_mb: 500,
            poll_interval_ac_mins: 5,
            poll_interval_battery_mins: 30,
            enable_realtime_hazard_alerts: true,
            database_name: "globalnews.db".into(),
        }
    }
}
