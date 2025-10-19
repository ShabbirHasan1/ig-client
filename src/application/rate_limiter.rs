/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/

//! Rate limiter module for controlling API request rates
//!
//! This module provides rate limiting functionality using the `governor` crate
//! to ensure compliance with IG Markets API rate limits.

use crate::application::config::RateLimiterConfig;
use governor::{
    Quota, RateLimiter as GovernorRateLimiter,
    clock::QuantaClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Rate limiter for controlling API request rates
///
/// Uses the `governor` crate to implement a token bucket algorithm
/// for rate limiting API requests.
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, QuantaClock>>,
}

impl RateLimiter {
    /// Creates a new rate limiter from configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Rate limiter configuration containing max requests, period, and burst size
    ///
    /// # Returns
    ///
    /// A new `RateLimiter` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// use ig_client::application::config::RateLimiterConfig;
    /// use ig_client::application::rate_limiter::RateLimiter;
    ///
    /// let config = RateLimiterConfig {
    ///     max_requests: 60,
    ///     period_seconds: 60,
    ///     burst_size: 10,
    /// };
    ///
    /// let limiter = RateLimiter::new(&config);
    /// ```
    #[must_use]
    pub fn new(config: &RateLimiterConfig) -> Self {
        let period = Duration::from_secs(config.period_seconds);

        let burst_size = NonZeroU32::new(config.burst_size)
            .unwrap_or_else(|| NonZeroU32::new(10).expect("10 is non-zero"));

        let quota = Quota::with_period(period)
            .expect("Valid period")
            .allow_burst(burst_size);

        let limiter = GovernorRateLimiter::direct(quota);

        Self {
            limiter: Arc::new(limiter),
        }
    }

    /// Waits until a request can be made according to the rate limit
    ///
    /// This method blocks until the rate limiter allows the request to proceed.
    /// It uses an async-friendly waiting mechanism.
    ///
    /// # Example
    ///
    /// ```ignore
    /// limiter.wait().await;
    /// // Make API request here
    /// ```
    pub async fn wait(&self) {
        while self.limiter.check().is_err() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Checks if a request can be made immediately without waiting
    ///
    /// # Returns
    ///
    /// * `true` if a request can be made immediately
    /// * `false` if the rate limit has been reached
    ///
    /// # Example
    ///
    /// ```ignore
    /// if limiter.check() {
    ///     // Make API request
    /// } else {
    ///     // Wait or handle rate limit
    /// }
    /// ```
    #[must_use]
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("limiter", &"GovernorRateLimiter")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let config = RateLimiterConfig {
            max_requests: 10,
            period_seconds: 1,
            burst_size: 5,
        };

        let limiter = RateLimiter::new(&config);

        // Should allow first few requests immediately
        for _ in 0..5 {
            assert!(limiter.check());
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_wait() {
        let config = RateLimiterConfig {
            max_requests: 2,
            period_seconds: 1,
            burst_size: 2,
        };

        let limiter = RateLimiter::new(&config);

        // First two requests should succeed immediately
        limiter.wait().await;
        limiter.wait().await;

        // Third request should wait
        let start = std::time::Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();

        // Should have waited some time (but not too long for the test)
        assert!(elapsed.as_millis() > 0);
    }
}
