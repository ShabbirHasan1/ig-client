use chrono::{Duration, Utc};
use ig_client::session::interface::{IgSession, TokenTimer};
use std::sync::Arc;
use std::thread;
use std::time::Duration as StdDuration;

#[test]
fn test_token_timer_new() {
    let timer = TokenTimer::new();
    let now = Utc::now();

    // Check that expiry is approximately 6 hours from now (within 1 minute tolerance)
    let expected_expiry = now + Duration::hours(6);
    let expiry_diff = (timer.expiry - expected_expiry).num_minutes().abs();
    assert!(
        expiry_diff <= 1,
        "Expiry should be ~6 hours from now, diff: {} minutes",
        expiry_diff
    );

    // Check that max_age is approximately 72 hours from now (within 1 minute tolerance)
    let expected_max_age = now + Duration::hours(72);
    let max_age_diff = (timer.max_age - expected_max_age).num_minutes().abs();
    assert!(
        max_age_diff <= 1,
        "Max age should be ~72 hours from now, diff: {} minutes",
        max_age_diff
    );

    // Check that last_refreshed is approximately now (within 1 minute tolerance)
    let refresh_diff = (timer.last_refreshed - now).num_minutes().abs();
    assert!(
        refresh_diff <= 1,
        "Last refreshed should be ~now, diff: {} minutes",
        refresh_diff
    );
}

#[test]
fn test_token_timer_is_expired_fresh_token() {
    let timer = TokenTimer::new();

    // Fresh token should not be expired
    assert!(!timer.is_expired(), "Fresh token should not be expired");
}

#[test]
fn test_token_timer_is_expired_expired_token() {
    let mut timer = TokenTimer::new();

    // Manually set expiry to past time
    timer.expiry = Utc::now() - Duration::hours(1);

    assert!(
        timer.is_expired(),
        "Token with past expiry should be expired"
    );
}

#[test]
fn test_token_timer_is_expired_max_age_exceeded() {
    let mut timer = TokenTimer::new();

    // Set max_age to past time (even if expiry is in future)
    timer.max_age = Utc::now() - Duration::hours(1);
    timer.expiry = Utc::now() + Duration::hours(1); // Future expiry

    assert!(
        timer.is_expired(),
        "Token with exceeded max_age should be expired"
    );
}

#[test]
fn test_token_timer_is_expired_w_margin_fresh_token() {
    let timer = TokenTimer::new();
    let margin = Duration::minutes(30);

    // Fresh token should not be expired even with margin
    assert!(
        !timer.is_expired_w_margin(margin),
        "Fresh token should not be expired with margin"
    );
}

#[test]
fn test_token_timer_is_expired_w_margin_within_margin() {
    let mut timer = TokenTimer::new();
    let margin = Duration::minutes(30);

    // Set expiry to 15 minutes from now (within 30 minute margin)
    timer.expiry = Utc::now() + Duration::minutes(15);

    assert!(
        timer.is_expired_w_margin(margin),
        "Token within margin should be considered expired"
    );
}

#[test]
fn test_token_timer_is_expired_w_margin_outside_margin() {
    let mut timer = TokenTimer::new();
    let margin = Duration::minutes(30);

    // Set expiry to 45 minutes from now (outside 30 minute margin)
    timer.expiry = Utc::now() + Duration::minutes(45);

    assert!(
        !timer.is_expired_w_margin(margin),
        "Token outside margin should not be considered expired"
    );
}

#[test]
fn test_token_timer_refresh() {
    let mut timer = TokenTimer::new();
    let original_expiry = timer.expiry;
    let original_last_refreshed = timer.last_refreshed;

    // Wait a small amount to ensure time difference
    thread::sleep(StdDuration::from_millis(10));

    timer.refresh();

    // Expiry should be updated to ~6 hours from now
    let now = Utc::now();
    let expected_expiry = now + Duration::hours(6);
    let expiry_diff = (timer.expiry - expected_expiry).num_minutes().abs();
    assert!(
        expiry_diff <= 1,
        "Refreshed expiry should be ~6 hours from now"
    );

    // Last refreshed should be updated
    assert!(
        timer.last_refreshed > original_last_refreshed,
        "Last refreshed should be updated"
    );

    // Expiry should be different from original
    assert!(
        timer.expiry != original_expiry,
        "Expiry should be updated after refresh"
    );
}

#[test]
fn test_ig_session_new_includes_token_timer() {
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Verify token_timer is properly initialized
    let timer = session.token_timer.lock().unwrap();
    let now = Utc::now();

    // Check that timer is properly initialized
    let expiry_diff = (timer.expiry - (now + Duration::hours(6)))
        .num_minutes()
        .abs();
    assert!(
        expiry_diff <= 1,
        "Session token timer should be properly initialized"
    );
}

#[test]
fn test_ig_session_refresh_token_timer() {
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    let original_expiry = {
        let timer = session.token_timer.lock().unwrap();
        timer.expiry
    };

    // Wait a small amount to ensure time difference
    thread::sleep(StdDuration::from_millis(10));

    // Refresh the token timer
    session.refresh_token_timer();

    let new_expiry = {
        let timer = session.token_timer.lock().unwrap();
        timer.expiry
    };

    // Expiry should be updated
    assert!(
        new_expiry > original_expiry,
        "Token timer expiry should be updated after refresh"
    );
}

#[test]
fn test_ig_session_refresh_token_timer_thread_safety() {
    let session = Arc::new(IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    ));

    let mut handles = vec![];

    // Spawn multiple threads that refresh the token timer concurrently
    for _ in 0..10 {
        let session_clone = Arc::clone(&session);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                session_clone.refresh_token_timer();
                thread::sleep(StdDuration::from_millis(1));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify the timer is still in a valid state
    let timer = session.token_timer.lock().unwrap();
    let now = Utc::now();

    // Timer should still be valid and not expired
    assert!(
        !timer.is_expired(),
        "Token should not be expired after concurrent refreshes"
    );

    // Expiry should be reasonable (within 6 hours + 1 minute from now)
    let expected_expiry = now + Duration::hours(6);
    let expiry_diff = (timer.expiry - expected_expiry).num_minutes().abs();
    assert!(
        expiry_diff <= 1,
        "Expiry should be reasonable after concurrent refreshes"
    );
}

#[test]
fn test_token_timer_edge_cases() {
    let mut timer = TokenTimer::new();

    // Test with zero margin
    let zero_margin = Duration::zero();
    assert!(
        !timer.is_expired_w_margin(zero_margin),
        "Fresh token should not be expired with zero margin"
    );

    // Test with negative margin (should still work)
    let negative_margin = Duration::minutes(-30);
    assert!(
        !timer.is_expired_w_margin(negative_margin),
        "Fresh token should not be expired with negative margin"
    );

    // Test refresh multiple times
    let original_expiry = timer.expiry;
    timer.refresh();
    let first_refresh = timer.expiry;
    thread::sleep(StdDuration::from_millis(10));
    timer.refresh();
    let second_refresh = timer.expiry;

    assert!(
        first_refresh > original_expiry,
        "First refresh should update expiry"
    );
    assert!(
        second_refresh >= first_refresh,
        "Second refresh should update or maintain expiry"
    );
}

#[test]
fn test_token_timer_realistic_scenario() {
    let mut timer = TokenTimer::new();

    // Simulate token usage over time
    // Token starts fresh
    assert!(!timer.is_expired(), "Token should start fresh");
    assert!(
        !timer.is_expired_w_margin(Duration::minutes(30)),
        "Token should not be near expiry initially"
    );

    // Simulate 5 hours passing (token still valid but getting close)
    timer.expiry = Utc::now() + Duration::hours(1); // 1 hour left
    assert!(
        !timer.is_expired(),
        "Token should still be valid with 1 hour left"
    );
    assert!(
        timer.is_expired_w_margin(Duration::minutes(90)),
        "Token should be considered expired with 90 minute margin"
    );

    // Refresh the token (simulating API usage)
    timer.refresh();
    assert!(!timer.is_expired(), "Token should be fresh after refresh");
    assert!(
        !timer.is_expired_w_margin(Duration::minutes(30)),
        "Refreshed token should not be near expiry"
    );

    // Simulate reaching max age
    timer.max_age = Utc::now() - Duration::minutes(1); // Max age exceeded
    assert!(
        timer.is_expired(),
        "Token should be expired when max age is exceeded"
    );
    assert!(
        timer.is_expired_w_margin(Duration::minutes(30)),
        "Token should be expired with margin when max age exceeded"
    );
}
