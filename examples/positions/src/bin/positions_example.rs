use ig_client::application::services::AccountService;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::account_service::AccountServiceImpl, config::Config,
    session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl, utils::finance::calculate_pnl,
    utils::logger::setup_logger,
};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Create configuration using the default Config implementation
    // This will read from environment variables as defined in src/config.rs
    // Using NonTradingAccount rate limit for demo accounts
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    info!("Configuration loaded → {}", config.rest_api.base_url);

    // Create HTTP client
    let http_client = Arc::new(IgHttpClientImpl::new(Arc::clone(&config)));
    info!("HTTP client created");

    // Create authenticator
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    // Login to IG and switch to the configured account if needed
    info!("Logging in to IG...");
    let session = if !config.credentials.account_id.trim().is_empty() {
        info!(
            "Using login_and_switch_account for account: {}",
            config.credentials.account_id
        );
        authenticator
            .login_and_switch_account(&config.credentials.account_id, Some(false))
            .await?
    } else {
        info!("Using standard login");
        authenticator.login().await?
    };
    info!(
        "Session started successfully for account: {}",
        session.account_id
    );

    // Create account service
    let account_service = AccountServiceImpl::new(Arc::clone(&config), Arc::clone(&http_client));
    info!("Account service created");

    // Get open positions
    info!("Fetching open positions...");
    let mut positions = account_service.get_positions(&session).await?;

    if positions.positions.is_empty() {
        info!("No open positions currently");
    } else {
        info!("Open positions: {}", positions.positions.len());

        // Display positions
        for (i, position) in positions.positions.iter_mut().enumerate() {
            // Calculate P&L using the utility function
            position.pnl = calculate_pnl(position);

            // Log the position as pretty JSON
            info!(
                "Position #{}: {}",
                i + 1,
                serde_json::to_string_pretty(&serde_json::to_value(position).unwrap()).unwrap()
            );
        }
    }

    // Get working orders
    info!("Fetching working orders...");
    let working_orders = account_service.get_working_orders(&session).await?;

    if working_orders.working_orders.is_empty() {
        info!("No working orders currently");
    } else {
        info!("Working orders: {}", working_orders.working_orders.len());

        // Display details of each working order as JSON
        for (i, order) in working_orders.working_orders.iter().enumerate() {
            // Log the working order as pretty JSON
            info!(
                "Working Order #{}: {}",
                i + 1,
                serde_json::to_string_pretty(&serde_json::to_value(order).unwrap()).unwrap()
            );
        }
    }

    Ok(())
}
