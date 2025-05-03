use std::time::Duration;
use std::env;

/// Server-side configuration for long polling
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct PollingConfig {
    /// Maximum time to keep a client connection open waiting for new data
    max_timeout: Duration,
    max_polled_items: usize,
}

impl PollingConfig {
    pub fn new() -> Self {
        // Try to get timeout from environment variable, default to 20 seconds
        let timeout_secs = env::var("POLLING_TIMEOUT")
            .ok()
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(20);

        let max_polled_items = env::var("POLL_ITEMS_COUNT")
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(5);

        Self {
            max_timeout: Duration::from_secs(timeout_secs),
            max_polled_items,
        }
    }

    /// Get the maximum timeout duration
    pub fn get_timeout(&self) -> Duration {
        self.max_timeout
    }

    pub fn get_max_polled_item(&self) -> usize {
        self.max_polled_items.clone()
    }
}