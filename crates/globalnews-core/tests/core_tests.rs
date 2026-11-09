use globalnews_core::*;

#[test]
fn test_core_engine_initialization_in_memory() {
    let engine = EngineState::new_in_memory().expect("In-memory engine initialization failed");
    assert_eq!(engine.config.database_name, "globalnews.db");
    assert_eq!(engine.config.storage_quota_mb, 500);
}

#[test]
fn test_benchmark_runner_execution() {
    let report = BenchmarkRunner::run_baseline();
    println!("Benchmark Report: {:?}", report);
    assert!(report.startup_time_ms > 0.0);
    assert!(report.db_init_time_ms > 0.0);
    assert!(report.serialization_ops_per_sec > 1000.0);
    assert!(report.deserialization_ops_per_sec > 1000.0);
    if let Some(rss) = report.memory_rss_mb {
        println!("Measured Memory RSS: {:.2} MB", rss);
        assert!(
            rss < 120.0,
            "Expected baseline engine memory to be well under 120 MB"
        );
    }
}
