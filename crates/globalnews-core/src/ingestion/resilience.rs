use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct BackoffPolicy {
    pub base_secs: u64,
    pub max_secs: u64,
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            base_secs: 30,
            max_secs: 3600, // 1 hour cap
        }
    }
}

impl BackoffPolicy {
    pub fn new(base_secs: u64, max_secs: u64) -> Self {
        Self { base_secs, max_secs }
    }

    /// Computes exponential backoff in seconds with deterministic pseudo-jitter.
    pub fn calculate_backoff_secs(&self, consecutive_failures: u32, retry_after_secs: Option<u64>) -> u64 {
        if let Some(ra) = retry_after_secs {
            return ra.min(self.max_secs);
        }

        if consecutive_failures == 0 {
            return 0;
        }

        let exp = (consecutive_failures - 1).min(6); // Prevent overflow (2^6 = 64)
        let backoff = self.base_secs.saturating_mul(1 << exp);

        // Simple jitter: + (failures % 5) seconds
        let jitter = (consecutive_failures as u64) % 5;
        (backoff + jitter).min(self.max_secs)
    }
}

/// Controls bounded concurrency across asynchronous feed fetch jobs.
#[derive(Clone)]
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
}

impl ConcurrencyLimiter {
    pub fn new(max_concurrent_requests: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent_requests)),
        }
    }

    pub async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.semaphore.clone().acquire_owned().await.unwrap()
    }
}

impl Default for ConcurrencyLimiter {
    fn default() -> Self {
        Self::new(4) // Conservative default: 4 concurrent fetches
    }
}
