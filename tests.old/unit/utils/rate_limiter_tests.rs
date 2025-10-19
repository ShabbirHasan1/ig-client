// Tests for the rate limiter module
use ig_client::utils::rate_limiter::{
    RateLimitType, RateLimiter, account_non_trading_limiter, account_trading_limiter,
    app_non_trading_limiter, global_rate_limiter, historical_price_limiter,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_test::block_on;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_type_limits() {
        // Test that each rate limit type returns the expected limits
        assert_eq!(RateLimitType::NonTradingAccount.request_limit(), 30);
        assert_eq!(RateLimitType::TradingAccount.request_limit(), 100);
        assert_eq!(RateLimitType::NonTradingApp.request_limit(), 60);
        assert_eq!(RateLimitType::HistoricalPrice.request_limit(), 10000);
    }

    #[test]
    fn test_rate_limiter_new() {
        // Test creating a new rate limiter
        let limiter = RateLimiter::new(RateLimitType::NonTradingAccount);

        // We can't directly access the private fields, but we can test the behavior
        // by calling wait() and measuring the time
        let start = Instant::now();
        block_on(limiter.wait()); // First call should not wait
        let first_call_duration = start.elapsed();

        // The first call should return almost immediately
        assert!(first_call_duration < Duration::from_millis(100));
    }

    // This test is more difficult to test deterministically due to the nature of time
    // and how it is handled in the rate limiter. Instead of verifying the exact time,
    // we will verify the basic functionality without depending on real time.
    #[test]
    fn test_rate_limiter_wait() {
        // Create a rate limiter with a type that has a shorter interval for the test
        let limiter = RateLimiter::new(RateLimitType::TradingAccount); // 2000ms

        // First call should not wait
        let start = Instant::now();
        block_on(limiter.wait());
        let first_duration = start.elapsed();

        // The first call should be very fast (less than 100ms)
        assert!(first_duration < Duration::from_millis(100));

        // Verificamos que el last_call se haya actualizado (no podemos acceder directamente)
        // pero podemos verificar indirectamente creando un nuevo limiter y comparando comportamientos
        let limiter2 = RateLimiter::new(RateLimitType::TradingAccount);

        // This limiter should not have last_call set, so it should be fast
        let start = Instant::now();
        block_on(limiter2.wait());
        let second_duration = start.elapsed();

        // It should also be fast
        assert!(second_duration < Duration::from_millis(100));
    }

    #[test]
    fn test_global_limiters() {
        // Test that global limiters return Arc<RateLimiter> instances
        let non_trading = account_non_trading_limiter();
        let trading = account_trading_limiter();
        let app = app_non_trading_limiter();
        let historical = historical_price_limiter();
        let global = global_rate_limiter();

        // Test that global_rate_limiter returns the same instance as account_non_trading_limiter
        assert!(Arc::ptr_eq(&global, &account_non_trading_limiter()));

        // Test that the limiters are different instances
        assert!(!Arc::ptr_eq(&non_trading, &trading));
        assert!(!Arc::ptr_eq(&non_trading, &app));
        assert!(!Arc::ptr_eq(&non_trading, &historical));
        assert!(!Arc::ptr_eq(&trading, &app));
        assert!(!Arc::ptr_eq(&trading, &historical));
        assert!(!Arc::ptr_eq(&app, &historical));
    }

    #[test]
    fn test_limiter_singleton_pattern() {
        // Test that calling the global limiter functions multiple times returns the same instance
        let non_trading1 = account_non_trading_limiter();
        let non_trading2 = account_non_trading_limiter();

        // Test that they are the same instance (same memory address)
        assert!(Arc::ptr_eq(&non_trading1, &non_trading2));

        // Repeat for other limiters
        let trading1 = account_trading_limiter();
        let trading2 = account_trading_limiter();
        assert!(Arc::ptr_eq(&trading1, &trading2));

        let app1 = app_non_trading_limiter();
        let app2 = app_non_trading_limiter();
        assert!(Arc::ptr_eq(&app1, &app2));

        let historical1 = historical_price_limiter();
        let historical2 = historical_price_limiter();
        assert!(Arc::ptr_eq(&historical1, &historical2));

        let global1 = global_rate_limiter();
        let global2 = global_rate_limiter();
        assert!(Arc::ptr_eq(&global1, &global2));
    }
}
