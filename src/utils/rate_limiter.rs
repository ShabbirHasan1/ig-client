// Rate limiter for API requests
// This module provides utilities to prevent hitting IG Markets API rate limits

use crate::constants::{BASE_DELAY_MS, SAFETY_BUFFER_MS};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Rate limiter type for different API endpoints with their respective limits
///
/// These limits are based on the official IG Markets API documentation:
/// - Per-app non-trading requests per minute: 60
/// - Per-account trading requests per minute: 100 (Applies to create/amend position or working order requests)
/// - Per-account non-trading requests per minute: 30
/// - Historical price data points per week: 10,000 (Applies to price history endpoints)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitType {
    /// Non-trading requests (per-account): 30 per minute
    NonTradingAccount,
    /// Trading requests (per-account): 100 per minute (create/amend position or working order requests)
    TradingAccount,
    /// Non-trading requests (per-app): 60 per minute
    NonTradingApp,
    /// Historical price data: 10,000 points per week (price history endpoints)
    HistoricalPrice,
    /// One request per second (special rate limit to prevent bursts)
    OnePerSecond,
}

impl RateLimitType {
    /// Gets the request limit per time window
    pub fn request_limit(&self) -> usize {
        match self {
            Self::NonTradingAccount => 30,  // 30 requests per minute
            Self::TradingAccount => 100,    // 100 requests per minute
            Self::NonTradingApp => 60,      // 60 requests per minute
            Self::HistoricalPrice => 10000, // 10,000 points per week
            Self::OnePerSecond => 1,        // 1 request per second
        }
    }

    /// Gets the time window in milliseconds
    pub fn time_window_ms(&self) -> u64 {
        match self {
            Self::NonTradingAccount => 60_000,    // 1 minute
            Self::TradingAccount => 60_000,       // 1 minute
            Self::NonTradingApp => 60_000,        // 1 minute
            Self::HistoricalPrice => 604_800_000, // 1 week
            Self::OnePerSecond => 1_000,          // 1 second
        }
    }

    /// Gets a description of the rate limit
    pub fn description(&self) -> String {
        match self {
            Self::NonTradingAccount => "30 requests per minute (per account)".to_string(),
            Self::TradingAccount => "100 requests per minute (per account)".to_string(),
            Self::NonTradingApp => "60 requests per minute (per app)".to_string(),
            Self::HistoricalPrice => "10,000 points per week".to_string(),
            Self::OnePerSecond => "1 request per second".to_string(),
        }
    }
}

/// Advanced rate limiter for API calls that maintains a request history
#[derive(Debug)]
pub struct RateLimiter {
    /// History of request timestamps
    request_history: Mutex<VecDeque<Instant>>,
    /// Type of rate limit to enforce
    limit_type: RateLimitType,
    /// Whether to apply a safety margin to the rate limit
    safety_margin: f64,
}

impl RateLimiter {
    /// Creates a new rate limiter with the specified limit type
    pub fn new(limit_type: RateLimitType) -> Self {
        RateLimiter {
            request_history: Mutex::new(VecDeque::new()),
            limit_type,
            safety_margin: 1.0,
        }
    }

    /// Creates a new rate limiter with a custom safety margin
    ///
    /// # Arguments
    ///
    /// * `safety_margin` - A value between 0.0 and 1.0 representing the percentage of the actual limit to use
    ///   (e.g., 0.8 means use 80% of the actual limit)
    pub fn with_safety_margin(&mut self, safety_margin: f64) -> Self {
        let safety_margin = safety_margin.clamp(0.1, 1.0);
        Self {
            request_history: Mutex::new(VecDeque::new()),
            limit_type: self.limit_type,
            safety_margin,
        }
    }

    /// Returns the rate limit type for this limiter
    pub fn limit_type(&self) -> RateLimitType {
        self.limit_type
    }

    /// Returns the effective request limit (after applying safety margin)
    pub fn effective_limit(&self) -> usize {
        let raw_limit = self.limit_type.request_limit();
        (raw_limit as f64 * self.safety_margin).floor() as usize
    }

    /// Removes expired requests from the history
    async fn cleanup_history(&self, now: Instant) {
        let mut history = self.request_history.lock().await;
        let window_duration = Duration::from_millis(self.limit_type.time_window_ms());

        // Remove requests that are older than the time window
        while let Some(oldest) = history.front() {
            if now.duration_since(*oldest) >= window_duration {
                history.pop_front();
            } else {
                break;
            }
        }
    }

    /// Gets the current number of requests in the time window
    pub async fn current_request_count(&self) -> usize {
        let history = self.request_history.lock().await;
        history.len()
    }

    /// Gets the time until the next request can be made (in milliseconds)
    /// Returns 0 if a request can be made immediately
    pub async fn time_until_next_request_ms(&self) -> u64 {
        let now = Instant::now();
        self.cleanup_history(now).await;

        // Use async lock to avoid blocking the thread
        let history = self.request_history.lock().await;
        let effective_limit = self.effective_limit();

        // Be more conservative: leave a safety margin for concurrent requests
        // This is especially important in recursive or concurrent contexts
        let usage_threshold = effective_limit.saturating_sub(2);

        if history.len() < usage_threshold {
            // We're well below the limit, no need to wait
            return 0;
        }

        // If we're close to the limit but haven't reached it, add a small delay
        // to prevent multiple concurrent requests from exceeding the limit
        if history.len() < effective_limit {
            // Add a small delay proportional to how close we are to the limit
            let proximity_factor = (history.len() as f64) / (effective_limit as f64);
            return (BASE_DELAY_MS as f64 * proximity_factor * proximity_factor).round() as u64;
        }

        // We're at the limit, need to wait for the oldest request to expire
        if let Some(oldest) = history.front() {
            let window_duration = Duration::from_millis(self.limit_type.time_window_ms());
            let time_since_oldest = now.duration_since(*oldest);

            if time_since_oldest < window_duration {
                // Calculate how long until the oldest request expires
                let wait_time = window_duration.saturating_sub(time_since_oldest);
                // Add a buffer for extra safety
                return wait_time.as_millis() as u64 + SAFETY_BUFFER_MS;
            }
        }

        0 // Should never reach here after cleanup, but just in case
    }

    /// Records a new request in the history
    async fn record_request(&self) {
        let now = Instant::now();
        let mut history = self.request_history.lock().await;
        history.push_back(now);
    }

    /// Notifies the rate limiter that a rate limit error has been encountered
    /// This will cause the rate limiter to enforce a mandatory cooldown period
    pub async fn notify_rate_limit_exceeded(&self) {
        // Add multiple "fake" requests to the history to force a cooldown
        let now = Instant::now();
        let mut history = self.request_history.lock().await;

        // Clear the history and add enough requests to reach the limit
        // This ensures we'll enforce a full cooldown period
        history.clear();

        // Add enough requests to reach the limit
        let limit = self.effective_limit();
        for _ in 0..limit {
            history.push_back(now);
        }

        warn!(
            "Rate limit exceeded! Enforcing mandatory cooldown period for {:?}",
            self.limit_type
        );
    }

    /// Waits if necessary to respect the rate limit
    /// This method is thread-safe and can be called from multiple threads concurrently
    pub async fn wait(&self) {
        // Register the request BEFORE waiting
        // This is crucial to prevent multiple concurrent requests from exceeding the rate limit
        self.record_request().await;

        // Now calculate the wait time based on the updated history
        let wait_time = self.time_until_next_request_ms().await;

        if wait_time > 0 {
            debug!(
                "Rate limiter ({:?}): waiting for {}ms ({}/{} requests used in window)",
                self.limit_type,
                wait_time,
                self.current_request_count().await,
                self.effective_limit()
            );
            sleep(Duration::from_millis(wait_time)).await;
        } else {
            debug!(
                "Rate limiter ({:?}): no wait needed ({}/{} requests used)",
                self.limit_type,
                self.current_request_count().await,
                self.effective_limit()
            );
        }
    }

    /// Gets statistics about the current rate limit usage
    pub async fn get_stats(&self) -> RateLimiterStats {
        let now = Instant::now();
        self.cleanup_history(now).await;

        let history = self.request_history.lock().await;
        let count = history.len();
        let limit = self.effective_limit();
        let usage_percent = if limit > 0 {
            (count as f64 / limit as f64) * 100.0
        } else {
            0.0
        };

        RateLimiterStats {
            limit_type: self.limit_type,
            request_count: count,
            effective_limit: limit,
            usage_percent,
        }
    }
}

/// Statistics about the rate limiter usage
#[derive(Debug)]
pub struct RateLimiterStats {
    /// Type of rate limit
    pub limit_type: RateLimitType,
    /// Current number of requests in the time window
    pub request_count: usize,
    /// Effective limit (raw limit * safety margin)
    pub effective_limit: usize,
    /// Usage percentage (current / effective limit)
    pub usage_percent: f64,
}

impl std::fmt::Display for RateLimiterStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RateLimiter({:?}): {}/{} requests ({:.1}%), window: {}ms",
            self.limit_type,
            self.request_count,
            self.effective_limit,
            self.usage_percent,
            self.limit_type.time_window_ms()
        )
    }
}

/// Global rate limiter for non-trading account requests (30 per minute)
pub fn account_non_trading_limiter() -> Arc<RateLimiter> {
    static INSTANCE: once_cell::sync::Lazy<Arc<RateLimiter>> = once_cell::sync::Lazy::new(|| {
        let mut limiter = RateLimiter::new(RateLimitType::NonTradingAccount);
        Arc::new(limiter.with_safety_margin(0.8))
    });

    INSTANCE.clone()
}

/// Global rate limiter for trading account requests (100 per minute)
pub fn account_trading_limiter() -> Arc<RateLimiter> {
    static INSTANCE: once_cell::sync::Lazy<Arc<RateLimiter>> = once_cell::sync::Lazy::new(|| {
        let mut limiter = RateLimiter::new(RateLimitType::TradingAccount);
        Arc::new(limiter.with_safety_margin(0.8))
    });

    INSTANCE.clone()
}

/// Global rate limiter for non-trading app requests (60 per minute)
pub fn app_non_trading_limiter() -> Arc<RateLimiter> {
    static INSTANCE: once_cell::sync::Lazy<Arc<RateLimiter>> = once_cell::sync::Lazy::new(|| {
        let mut limiter = RateLimiter::new(RateLimitType::NonTradingApp);
        Arc::new(limiter.with_safety_margin(0.8))
    });

    INSTANCE.clone()
}

/// Global rate limiter for historical price data requests (10,000 points per week)
pub fn historical_price_limiter() -> Arc<RateLimiter> {
    static INSTANCE: once_cell::sync::Lazy<Arc<RateLimiter>> = once_cell::sync::Lazy::new(|| {
        let mut limiter = RateLimiter::new(RateLimitType::HistoricalPrice);
        Arc::new(limiter.with_safety_margin(0.8))
    });

    INSTANCE.clone()
}

/// Creates a rate limiter with the specified type
pub fn create_rate_limiter(
    limit_type: RateLimitType,
    safety_margin: Option<f64>,
) -> Arc<RateLimiter> {
    let mut limiter = RateLimiter::new(limit_type);
    match safety_margin {
        Some(margin) => Arc::new(limiter.with_safety_margin(margin)),
        None => Arc::new(limiter),
    }
}

/// Global rate limiter for one request per second
pub fn one_per_second_limiter() -> Arc<RateLimiter> {
    static INSTANCE: once_cell::sync::Lazy<Arc<RateLimiter>> = once_cell::sync::Lazy::new(|| {
        let limiter = RateLimiter::new(RateLimitType::OnePerSecond);
        Arc::new(limiter)
    });

    INSTANCE.clone()
}

/// Default global rate limiter (uses the most conservative limit: non-trading account)
pub fn global_rate_limiter() -> Arc<RateLimiter> {
    account_non_trading_limiter()
}

/// Macro to mark tests that should be run individually to avoid rate limiting
#[macro_export]
macro_rules! rate_limited_test {
    (fn $name:ident() $body:block) => {
        #[test]
        #[ignore]
        fn $name() $body
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_rate_limiter_effective_limit() {
        let limiter = RateLimiter::new(RateLimitType::NonTradingAccount);
        assert_eq!(limiter.effective_limit(), 30); // Default safety margin is 1.0

        let mut limiter = RateLimiter::new(RateLimitType::NonTradingAccount);
        let limiter = limiter.with_safety_margin(0.5);
        assert_eq!(limiter.effective_limit(), 15); // 30 * 0.5 = 15
    }

    #[test]
    fn test_rate_limiter_history_tracking() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut limiter = RateLimiter::new(RateLimitType::NonTradingAccount);
            let limiter = limiter.with_safety_margin(1.0);
            assert_eq!(limiter.current_request_count().await, 0);

            // Record some requests manually
            for _ in 0..5 {
                limiter.record_request().await;
            }

            assert_eq!(limiter.current_request_count().await, 5);
        });
    }

    #[test]
    fn test_rate_limiter_stats() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let mut limiter = RateLimiter::new(RateLimitType::NonTradingAccount);
            let limiter = limiter.with_safety_margin(0.8);

            // Record some requests
            for _ in 0..10 {
                limiter.record_request().await;
            }

            let stats = limiter.get_stats().await;
            assert_eq!(stats.request_count, 10);
            assert_eq!(stats.effective_limit, 24); // 30 * 0.8 = 24
            assert!(stats.usage_percent > 0.0);
        });
    }
}
