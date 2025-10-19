use crate::common;
use ig_client::session::interface::IgAuthenticator;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter;
use tokio::runtime::Runtime;
use tracing::info;

#[test]
#[ignore]
fn test_login() {
    setup_logger();
    // Get a session
    let session = common::login_with_account_switch();

    // Verify the session contains the expected fields
    assert!(!session.cst.is_empty(), "CST token should not be empty");
    assert!(
        !session.token.is_empty(),
        "Security token should not be empty"
    );
    assert!(
        !session.account_id.is_empty(),
        "Account ID should not be empty"
    );

    info!("Login successful. Account ID: {}", session.account_id);
}

// This test is marked as ignored to avoid hitting the rate limit
// Run with: cargo test --test integration_tests -- --ignored test_account_switch
#[test]
#[ignore]
fn test_account_switch() {
    setup_logger();
    // Skip this test if no account ID is specified in the config
    let config = common::create_test_config();
    if config.credentials.account_id.is_empty() {
        info!("Skipping account switch test as no account ID is specified in the config");
        return;
    }

    // Wait to respect the rate limit
    let rt = Runtime::new().expect("Failed to create runtime");
    rt.block_on(async {
        rate_limiter::global_rate_limiter().wait().await;
    });

    // Create test configuration and authenticator
    let auth = common::create_test_auth(&config);

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test login and account switch
    rt.block_on(async {
        // Login first
        let session = auth.login().await.expect("Failed to login");

        // Skip the test if we're already on the target account
        if session.account_id == config.credentials.account_id {
            info!(
                "Already on the target account: {}. Skipping switch.",
                session.account_id
            );
            return;
        }

        // Switch to the specified account
        info!("Switching to account: {}", config.credentials.account_id);
        let new_session = auth
            .switch_account(&session, &config.credentials.account_id, Some(true))
            .await
            .expect("Failed to switch account");

        // Verify the account switch was successful
        assert_eq!(
            new_session.account_id, config.credentials.account_id,
            "Account switch did not result in the expected account ID"
        );

        info!(
            "Successfully switched to account: {}",
            new_session.account_id
        );
    });
}
