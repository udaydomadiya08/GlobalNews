mod commands;

use commands::*;
use globalnews_core::EngineState;
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting GlobalNews Desktop / Mobile Application Shell");

    let engine = EngineState::new_desktop().expect("Failed to initialize desktop engine state");
    let health = handle_get_health(&engine);

    info!(
        "GlobalNews Shell Ready: platform='{}', schema='{}', status='{}'",
        health.platform, health.schema_version, health.status
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_process_command_handling() {
        let engine = EngineState::new_in_memory().unwrap();
        let health = handle_get_health(&engine);
        assert_eq!(health.status, "OK");
        assert_eq!(health.schema_version, "1.0.0");
        assert!(health.database_active);
    }
}
