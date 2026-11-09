use crate::state::EngineState;
use globalnews_contracts::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BenchmarkReport {
    pub startup_time_ms: f64,
    pub db_init_time_ms: f64,
    pub serialization_ops_per_sec: f64,
    pub deserialization_ops_per_sec: f64,
    pub memory_rss_mb: Option<f64>,
}

pub struct BenchmarkRunner;

impl BenchmarkRunner {
    pub fn run_baseline() -> BenchmarkReport {
        let t0 = Instant::now();

        // 1. Measure DB & Engine Startup Time
        let db_t0 = Instant::now();
        let engine = EngineState::new_in_memory().expect("Engine startup failed");
        let db_init_time_ms = db_t0.elapsed().as_secs_f64() * 1000.0;
        let startup_time_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // 2. Measure Serialization & Deserialization Throughput
        let sample_event = MicroEvent {
            id: "bench-001".into(),
            macro_event_id: None,
            title: "Global Benchmark Sample Event".into(),
            summary: "Measuring serialization and deserialization throughput for contracts.".into(),
            category_code: "03000000".into(),
            importance: ImportanceScore::new(8.0, 7.0, 6.0, 5).unwrap(),
            confidence: ConfidenceScore::new(5, 0.85, true, false).unwrap(),
            location: GeographicLocation::new(0.0, 0.0, "Null Island", "XX", 1.0).unwrap(),
            status: EventStatus::Confirmed,
            first_seen_at: "2026-11-09T00:00:00Z".into(),
            last_updated_at: "2026-11-09T01:00:00Z".into(),
            citation_count: 5,
        };

        let iterations = 10_000;

        // Serialization benchmark
        let ser_t0 = Instant::now();
        let mut last_json = String::new();
        for _ in 0..iterations {
            last_json = serde_json::to_string(&sample_event).unwrap();
        }
        let ser_duration = ser_t0.elapsed().as_secs_f64();
        let serialization_ops_per_sec = iterations as f64 / ser_duration;

        // Deserialization benchmark
        let de_t0 = Instant::now();
        for _ in 0..iterations {
            let _: MicroEvent = serde_json::from_str(&last_json).unwrap();
        }
        let de_duration = de_t0.elapsed().as_secs_f64();
        let deserialization_ops_per_sec = iterations as f64 / de_duration;

        // 3. Inspect Memory RSS if available on platform
        let memory_rss_mb = Self::get_memory_rss_mb();

        let _ = engine; // Keep alive

        BenchmarkReport {
            startup_time_ms,
            db_init_time_ms,
            serialization_ops_per_sec,
            deserialization_ops_per_sec,
            memory_rss_mb,
        }
    }

    #[cfg(unix)]
    fn get_memory_rss_mb() -> Option<f64> {
        let mut rusage = std::mem::MaybeUninit::uninit();
        let ret = unsafe { libc::getrusage(libc::RUSAGE_SELF, rusage.as_mut_ptr()) };
        if ret == 0 {
            let usage = unsafe { rusage.assume_init() };
            #[cfg(target_os = "macos")]
            let bytes = usage.ru_maxrss as f64; // macOS: bytes
            #[cfg(not(target_os = "macos"))]
            let bytes = (usage.ru_maxrss as f64) * 1024.0; // Linux: kilobytes
            Some(bytes / (1024.0 * 1024.0))
        } else {
            None
        }
    }

    #[cfg(not(unix))]
    fn get_memory_rss_mb() -> Option<f64> {
        None
    }
}
