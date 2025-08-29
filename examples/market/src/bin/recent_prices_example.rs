use ig_client::application::services::{MarketService, RecentPricesParams};
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::market_service::MarketServiceImpl, config::Config,
    session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl, utils::logger::setup_logger,
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Create configuration
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));

    info!("=== IG Client Recent Prices Example (API v3) ===");
    info!("Configuration loaded:");
    info!("  Base URL: {}", config.rest_api.base_url);
    info!("  Username: {}", config.credentials.username);

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

    // Create HTTP client
    let http_client = Arc::new(IgHttpClientImpl::new(Arc::clone(&config)));
    info!("HTTP client created");

    // Create market service
    let market_service = MarketServiceImpl::new(Arc::clone(&config), Arc::clone(&http_client));
    info!("Market service created");

    // Create authenticator
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    // Login to IG
    info!("Attempting to login to IG...");
    let session = match authenticator.login().await {
        Ok(session) => {
            info!("✅ Login successful. Account: {}", session.account_id);
            session
        }
        Err(e) => {
            error!("❌ Login failed: {:?}", e);
            return Err("Login to IG failed".into());
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

    // Find a market to test with
    info!("\n=== Finding a Market for Testing ===");
    let search_results = market_service
        .search_markets(&session, "Daily Germany")
        .await?;

    let epic = if let Some(market) = search_results.markets.first() {
        info!(
            "✅ Found market: {} ({})",
            market.instrument_name, market.epic
        );
        market.epic.clone()
    } else {
        error!("❌ No markets found for Daily Germany");
        return Err("No test market available".into());
    };

    info!("\n=== Testing Recent Prices Endpoint (API v3) ===");
    info!("Endpoint: /prices/{{epic}}");
    info!("Description: Returns recent minute prices within the last 10 minutes by default");

    // Example 1: Default parameters (minute prices, last 10 minutes)
    info!("\n🔍 Example 1: Default parameters");
    let params = RecentPricesParams::new(&epic);
    match market_service.get_recent_prices(&session, &params).await {
        Ok(prices) => {
            info!(
                "✅ Recent prices obtained: {} data points",
                prices.prices.len()
            );
            info!("  Instrument type: {:?}", prices.instrument_type);

            if let Some(allowance) = &prices.allowance {
                info!("  📊 API Allowance:");
                info!("    Remaining: {}", allowance.remaining_allowance);
                info!("    Total: {}", allowance.total_allowance);
                info!("    Expires in: {} seconds", allowance.allowance_expiry);
            }

            if let Some(first_price) = prices.prices.first() {
                info!("  📈 Latest price data:");
                info!("    Time: {}", first_price.snapshot_time);
                info!(
                    "    Close - Bid: {:?}, Ask: {:?}",
                    first_price.close_price.bid, first_price.close_price.ask
                );
                if let Some(volume) = first_price.last_traded_volume {
                    info!("    Volume: {}", volume);
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get recent prices: {:?}", e);
        }
    }

    // Example 2: Custom resolution (5-minute candles)
    info!("\n🔍 Example 2: 5-minute resolution with limited data points");
    let params = RecentPricesParams::new(&epic)
        .with_resolution("MINUTE_5")
        .with_max_points(5);
    match market_service.get_recent_prices(&session, &params).await {
        Ok(prices) => {
            info!(
                "✅ 5-minute prices obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }

            for (i, price) in prices.prices.iter().enumerate() {
                info!(
                    "  📊 Candle {}: {} - OHLC(Bid): O:{:?} H:{:?} L:{:?} C:{:?}",
                    i + 1,
                    price.snapshot_time,
                    price.open_price.bid,
                    price.high_price.bid,
                    price.low_price.bid,
                    price.close_price.bid
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get 5-minute prices: {:?}", e);
        }
    }

    // Example 3: Hourly resolution with pagination
    info!("\n🔍 Example 3: Hourly resolution with pagination");
    let params = RecentPricesParams::new(&epic)
        .with_resolution("HOUR")
        .with_max_points(3)
        .with_page_size(5)
        .with_page_number(1);
    match market_service.get_recent_prices(&session, &params).await {
        Ok(prices) => {
            info!(
                "✅ Hourly prices obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }

            for (i, price) in prices.prices.iter().enumerate() {
                info!("  🕐 Hour {}: {}", i + 1, price.snapshot_time);
                info!(
                    "    Bid prices - Open: {:?}, High: {:?}, Low: {:?}, Close: {:?}",
                    price.open_price.bid,
                    price.high_price.bid,
                    price.low_price.bid,
                    price.close_price.bid
                );
                info!(
                    "    Ask prices - Open: {:?}, High: {:?}, Low: {:?}, Close: {:?}",
                    price.open_price.ask,
                    price.high_price.ask,
                    price.low_price.ask,
                    price.close_price.ask
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get hourly prices: {:?}", e);
        }
    }

    // Example 4: With specific time range (if supported)
    info!("\n🔍 Example 4: With specific time range");
    let params = RecentPricesParams::new(&epic)
        .with_resolution("MINUTE_15")
        .with_from("2024-01-01T10:00:00")
        .with_to("2024-01-01T12:00:00");
    match market_service.get_recent_prices(&session, &params).await {
        Ok(prices) => {
            info!(
                "✅ Time range prices obtained: {} data points",
                prices.prices.len()
            );

            if !prices.prices.is_empty() {
                info!(
                    "  📅 Time range: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );
            }

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get time range prices: {:?}", e);
            info!("  💡 Note: Time range parameters might not be supported for recent data");
        }
    }

    info!("\n=== Summary ===");
    info!("📋 Recent Prices Endpoint (API v3) Features:");
    info!("  • Endpoint: /prices/{{epic}}");
    info!("  • Default: Minute prices from last 10 minutes");
    info!("  • Flexible parameters: resolution, from, to, max, pageSize, pageNumber");
    info!("  • Supported resolutions: SECOND, MINUTE, MINUTE_2, MINUTE_3, MINUTE_5,");
    info!("    MINUTE_10, MINUTE_15, MINUTE_30, HOUR, HOUR_2, HOUR_3, HOUR_4, DAY, WEEK, MONTH");
    info!("  • Pagination support for large datasets");
    info!("  • Real-time allowance tracking");
    info!("  • Ideal for: Recent market data, real-time monitoring, flexible queries");

    info!("\n=== Example completed successfully! ===");

    Ok(())
}
