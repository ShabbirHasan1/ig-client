use ig_client::application::services::AccountService;
/// Example demonstrating how to get positions from one account, switch to another account,
/// and get positions from the new account
///
/// This example shows:
/// 1. Login to the API
/// 2. Get positions from the current account
/// 3. Switch to a different account (default: ZHJ5N, or specify via command line)
/// 4. Get positions from the new account
///
/// To run this example:
/// ```bash
/// # With default account (ZHJ5N)
/// cargo run --bin positions_switch_account
///
/// # With a specific account
/// cargo run --bin positions_switch_account YOUR_ACCOUNT_ID
/// ```
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::config::Config;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::transport::http_client::IgHttpClientImpl;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use std::error::Error;
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    setup_logger();

    info!("=== Positions Switch Account Example ===\n");

    // Create configuration
    let mut config = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.7);
    // Use API v2 (CST) - account switching doesn't work with OAuth on demo API
    config.api_version = Some(3);
    let config = Arc::new(config);

    info!("Configuration loaded:");
    info!("  Base URL: {}", config.rest_api.base_url);
    info!("  API Version: {:?}", config.api_version);

    // Create HTTP client and services
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));
    let account_service = AccountServiceImpl::new(config.clone(), client);
    let auth = IgAuth::new(&config);

    // Step 1: Login
    info!("\n1. Logging in...");
    let session = match auth.login().await {
        Ok(s) => s,
        Err(e) => {
            error!("✗ Login failed: {:?}", e);
            return Err(format!("Login error: {:?}", e).into());
        }
    };

    info!("✓ Login successful");
    info!("  Account ID: {}", session.account_id);
    info!("  Uses OAuth: {}", session.is_oauth());

    // Step 2: Get positions from current account
    info!(
        "\n2. Getting positions from account: {}",
        session.account_id
    );
    match account_service.get_positions(&session).await {
        Ok(positions) => {
            info!("✓ Successfully retrieved positions");
            info!("  Total positions: {}", positions.positions.len());

            if positions.positions.is_empty() {
                info!("  No open positions in this account");
            } else {
                info!("\n  Open positions:");
                for (i, position) in positions.positions.iter().enumerate() {
                    info!(
                        "  {}. {} - {} @ {} (Size: {})",
                        i + 1,
                        position.market.instrument_name,
                        position.market.epic,
                        position.position.direction,
                        position.position.size
                    );
                }
            }
        }
        Err(e) => {
            error!("✗ Failed to get positions: {:?}", e);
            return Err(format!("Get positions error (account 1): {:?}", e).into());
        }
    }

    // Step 3: Switch to target account (from command line or default to ZHJ5N)
    let target_account = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "ZHJ5N".to_string());

    info!("\n3. Switching to account: {}", target_account);
    info!(
        "   (You can specify a different account as: cargo run --bin positions_switch_account <ACCOUNT_ID>)"
    );

    let new_session = match auth
        .switch_account(&session, &target_account, Some(false))
        .await
    {
        Ok(s) => {
            info!("✓ Successfully switched to account: {}", s.account_id);
            s
        }
        Err(e) => {
            error!("✗ Failed to switch account: {:?}", e);
            error!("  This could happen if:");
            error!("  - The account ID doesn't exist");
            error!("  - You don't have permission to access this account");
            error!("  - The account is not associated with your user");
            return Err(format!("Account switch error: {:?}", e).into());
        }
    };

    // Step 4: Get positions from new account
    info!(
        "\n4. Getting positions from account: {}",
        new_session.account_id
    );
    match account_service.get_positions(&new_session).await {
        Ok(positions) => {
            info!("✓ Successfully retrieved positions");
            info!("  Total positions: {}", positions.positions.len());

            if positions.positions.is_empty() {
                info!("  No open positions in this account");
            } else {
                info!("\n  Open positions:");
                for (i, position) in positions.positions.iter().enumerate() {
                    info!(
                        "  {}. {} - {} @ {} (Size: {})",
                        i + 1,
                        position.market.instrument_name,
                        position.market.epic,
                        position.position.direction,
                        position.position.size
                    );
                }
            }
        }
        Err(e) => {
            error!("✗ Failed to get positions: {:?}", e);
            return Err(format!("Get positions error (account 2): {:?}", e).into());
        }
    }

    info!("\n=== Example Complete ===");
    Ok(())
}
