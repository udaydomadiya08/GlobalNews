pub mod benchmarks;
pub mod config;
pub mod error;
pub mod ingestion;
pub mod state;

pub use benchmarks::{BenchmarkReport, BenchmarkRunner};
pub use config::CoreConfig;
pub use error::CoreError;
pub use state::EngineState;
