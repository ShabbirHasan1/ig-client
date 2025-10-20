// Integration tests for account endpoints

use crate::common;
use ig_client::prelude::*;
use tokio::runtime::Runtime;
use tracing::info;

#[test]
#[ignore]
fn test_get_accounts() {
    setup_logger();
    // Create client
    let client = common::create_test_client();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get accounts
    rt.block_on(async {
        info!("Getting accounts");

        let result = client.get_accounts().await.expect("Failed to get accounts");

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
    // Create client
    let client = common::create_test_client();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get account activity
    rt.block_on(async {
        // Use a date range for the last 7 days
        use chrono::{Duration, Utc};

        let to = Utc::now();
        let from = to - Duration::days(7);

        let from_str = from.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        info!("Getting account activity from {} to {}", from_str, to_str);

        let result = client
            .get_activity(&from_str, &to_str)
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
    // Create client
    let client = common::create_test_client();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get transaction history
    rt.block_on(async {
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

        let result = client
            .get_transactions(&from_str, &to_str)
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
