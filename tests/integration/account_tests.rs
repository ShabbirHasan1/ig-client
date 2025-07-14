// Integration tests for account endpoints

use crate::common;
use ig_client::utils::logger::setup_logger;
use ig_client::{
    application::services::AccountService,
    application::services::account_service::AccountServiceImpl,
};
use tokio::runtime::Runtime;
use tracing::info;

#[test]
#[ignore]
fn test_get_accounts() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create account service
    let account_service = AccountServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get accounts
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per account)
        ig_client::utils::rate_limiter::account_non_trading_limiter()
            .wait()
            .await;
        info!("Getting accounts");

        let result = account_service
            .get_accounts(&session)
            .await
            .expect("Failed to get accounts");

        // Verify the result contains the expected data
        assert!(
            !result.accounts.is_empty(),
            "Should return at least one account"
        );

        info!("Retrieved {} accounts", result.accounts.len());

        // Print the accounts
        for (i, account) in result.accounts.iter().enumerate() {
            info!(
                "{}. {} (ID: {})",
                i + 1,
                account.account_name,
                account.account_id
            );
            info!(
                "   Type: {}, Status: {}",
                account.account_type, account.status
            );
            info!("   Currency: {}", account.currency);
            info!(
                "   Can trade: {}",
                if account.preferred { "Yes" } else { "No" }
            );
        }
    });
}

#[test]
#[ignore]
fn test_get_account_activity() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create account service
    let account_service = AccountServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get account activity
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per account)
        ig_client::utils::rate_limiter::account_non_trading_limiter()
            .wait()
            .await;
        // Use a date range for the last 7 days
        use chrono::{Duration, Utc};

        let to = Utc::now();
        let from = to - Duration::days(7);

        let from_str = from.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        info!("Getting account activity from {} to {}", from_str, to_str);

        let result = account_service
            .get_activity(&session, &from_str, &to_str)
            .await
            .expect("Failed to get account activity");

        // Print the activities
        info!("Retrieved {} account activities", result.activities.len());

        if result.activities.is_empty() {
            info!("No activities found in the specified date range");
        } else {
            for (i, activity) in result.activities.iter().enumerate() {
                info!(
                    "{}.  {:?} on {}",
                    i + 1,
                    activity.activity_type,
                    activity.date
                );
                info!("   Details: {:?}", activity.details);
                info!("   Channel: {:?}", activity.channel);
                info!("   Status: {:?}", activity.status);
            }
        }
    });
}

#[test]
#[ignore]
fn test_get_transaction_history() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create account service
    let account_service = AccountServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get transaction history
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per account)
        ig_client::utils::rate_limiter::account_non_trading_limiter()
            .wait()
            .await;
        // Use a date range for the last 30 days
        use chrono::{Duration, Utc};

        let to = Utc::now();
        let from = to - Duration::days(30);

        let from_str = from.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        info!(
            "Getting transaction history from {} to {}",
            from_str, to_str
        );

        let result = account_service
            .get_transactions(&session, &from_str, &to_str)
            .await
            .expect("Failed to get transaction history");

        // Print the transactions
        info!("Retrieved {} transactions", result.transactions.len());

        if result.transactions.is_empty() {
            info!("No transactions found in the specified date range");
        } else {
            for (i, transaction) in result.transactions.iter().enumerate() {
                info!(
                    "{}. {} on {}",
                    i + 1,
                    transaction.transaction_type,
                    transaction.date
                );
                info!("   Instrument: {}", transaction.instrument_name);
                info!("   Reference: {}", transaction.reference);
                info!(
                    "   Amount: {} ({})",
                    transaction.profit_and_loss, transaction.currency
                );
            }
        }
    });
}
