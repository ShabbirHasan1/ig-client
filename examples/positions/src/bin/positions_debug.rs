use ig_client::application::services::AccountService;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::account_service::AccountServiceImpl, config::Config,
    session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl, utils::finance::calculate_pnl,
    utils::logger::setup_logger,
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Create configuration using the default Config implementation
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));

    info!("=== IG Client Positions Debug Example ===");
    info!("Configuration loaded:");
    info!("  Base URL: {}", config.rest_api.base_url);
    info!("  Username: {}", config.credentials.username);
    info!("  Account ID: {}", config.credentials.account_id);
    info!("  API Key length: {}", config.credentials.api_key.len());
    info!("  Rate limit type: {:?}", config.rate_limit_type);

    // Validate configuration
    if config.credentials.username.is_empty() {
        error!("❌ Username is empty. Please set IG_USERNAME environment variable.");
        return Err("Missing username configuration".into());
    }

    if config.credentials.password.is_empty() {
        error!("❌ Password is empty. Please set IG_PASSWORD environment variable.");
        return Err("Missing password configuration".into());
    }

    if config.credentials.api_key.is_empty() {
        error!("❌ API key is empty. Please set IG_API_KEY environment variable.");
        return Err("Missing API key configuration".into());
    }

    // Check if using demo or live environment
    let is_demo = config.rest_api.base_url.contains("demo");
    info!("Environment: {}", if is_demo { "DEMO" } else { "LIVE" });

    if !is_demo {
        warn!("⚠️  You are connecting to the LIVE environment!");
    }

    // Create HTTP client
    let http_client = Arc::new(IgHttpClientImpl::new(Arc::clone(&config)));
    info!("HTTP client created");

    // Create authenticator
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    // Login to IG with detailed error handling
    info!("Attempting to login to IG...");
    let session = match authenticator.login().await {
        Ok(session) => {
            info!("✅ Login successful!");
            info!("  Account ID: {}", session.account_id);
            info!("  CST token length: {}", session.cst.len());
            info!("  Security token length: {}", session.token.len());
            session
        }
        Err(e) => {
            error!("❌ Login failed: {:?}", e);
            error!("Common causes:");
            error!("  1. Invalid credentials (username/password/API key)");
            error!("  2. Account locked or suspended");
            error!("  3. API key not activated or expired");
            error!("  4. Wrong environment (demo vs live)");
            error!("  5. Rate limiting (too many requests)");
            error!("  6. Network connectivity issues");

            // Provide specific guidance based on error type
            let error_str = format!("{:?}", e);
            if error_str.contains("FORBIDDEN") || error_str.contains("403") {
                error!("🔍 403 Forbidden suggests:");
                error!("   - Check your username and password are correct");
                error!("   - Verify your API key is valid and activated");
                error!("   - Ensure you're using the right environment (demo/live)");
                error!("   - Check if your account has API access enabled");
            } else if error_str.contains("UNAUTHORIZED") || error_str.contains("401") {
                error!("🔍 401 Unauthorized suggests:");
                error!("   - Invalid API key or credentials");
                error!("   - API key might be expired or deactivated");
            } else if error_str.contains("TOO_MANY_REQUESTS") || error_str.contains("429") {
                error!("🔍 429 Too Many Requests suggests:");
                error!("   - You've hit the rate limit");
                error!("   - Wait a few minutes before trying again");
            }

            return Err(e.into());
        }
    };

    // Switch to configured account if different
    let session = if !config.credentials.account_id.trim().is_empty()
        && session.account_id != config.credentials.account_id
    {
        info!(
            "Switching to configured account: {}",
            config.credentials.account_id
        );
        match authenticator
            .switch_account(&session, &config.credentials.account_id, Some(false))
            .await
        {
            Ok(new_session) => {
                info!(
                    "✅ Account switch successful to: {}",
                    new_session.account_id
                );
                new_session
            }
            Err(e) => {
                warn!("⚠️  Account switch failed: {:?}", e);
                warn!("Continuing with original account: {}", session.account_id);
                session
            }
        }
    } else {
        info!("Using account: {}", session.account_id);
        session
    };

    // Create account service
    let account_service = AccountServiceImpl::new(Arc::clone(&config), Arc::clone(&http_client));
    info!("Account service created");

    // Get open positions with error handling
    info!("\n=== Fetching Open Positions ===");
    match account_service.get_positions(&session).await {
        Ok(mut positions) => {
            if positions.positions.is_empty() {
                info!("📊 No open positions currently");
            } else {
                info!("📊 Found {} open position(s)", positions.positions.len());

                // Display positions
                for (i, position) in positions.positions.iter_mut().enumerate() {
                    // Calculate P&L using the utility function
                    position.pnl = calculate_pnl(position);

                    info!("\n--- Position #{} ---", i + 1);
                    info!("Epic: {}", position.market.epic);
                    info!("Instrument: {}", position.market.instrument_name);
                    info!("Direction: {:?}", position.position.direction);
                    info!("Size: {}", position.position.size);
                    info!("Level: {}", position.position.level);
                    info!("P&L: {:.2}", position.pnl.unwrap_or(0.0));
                    info!("Currency: {}", position.position.currency);

                    // Log full position as JSON for debugging
                    info!(
                        "Full position data: {}",
                        serde_json::to_string_pretty(&serde_json::to_value(position).unwrap())
                            .unwrap()
                    );
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch positions: {:?}", e);
            error!("This could be due to:");
            error!("  - Session expired (try re-running)");
            error!("  - Network connectivity issues");
            error!("  - API rate limiting");
            error!("  - Account permissions");
        }
    }

    // Get working orders with error handling
    info!("\n=== Fetching Working Orders ===");
    match account_service.get_working_orders(&session).await {
        Ok(working_orders) => {
            if working_orders.working_orders.is_empty() {
                info!("📋 No working orders currently");
            } else {
                info!(
                    "📋 Found {} working order(s)",
                    working_orders.working_orders.len()
                );

                // Display details of each working order
                for (i, order) in working_orders.working_orders.iter().enumerate() {
                    info!("\n--- Working Order #{} ---", i + 1);
                    info!("Epic: {}", order.market_data.epic);
                    info!("Instrument: {}", order.market_data.instrument_name);
                    info!("Direction: {:?}", order.working_order_data.direction);
                    info!("Size: {}", order.working_order_data.order_size);
                    info!("Order Level: {}", order.working_order_data.order_level);
                    info!("Order Type: {:?}", order.working_order_data.order_type);
                    info!(
                        "Time in Force: {:?}",
                        order.working_order_data.time_in_force
                    );

                    // Log full order as JSON for debugging
                    info!(
                        "Full order data: {}",
                        serde_json::to_string_pretty(&serde_json::to_value(order).unwrap())
                            .unwrap()
                    );
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch working orders: {:?}", e);
            error!("This could be due to:");
            error!("  - Session expired (try re-running)");
            error!("  - Network connectivity issues");
            error!("  - API rate limiting");
            error!("  - Account permissions");
        }
    }

    info!("\n=== Example completed successfully! ===");
    Ok(())
}
