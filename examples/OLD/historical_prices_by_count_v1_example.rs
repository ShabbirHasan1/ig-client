use ig_client::application::services::MarketService;
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
    let mut config = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.7);
    // Use API v3 (OAuth) by default
    config.api_version = Some(3);
    let config = Arc::new(config);

    info!("=== IG Client Historical Prices by Count Example (API v1) ===");
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
            info!("✅ Login successful!");
            info!("  Account ID: {}", session.account_id);
            session
        }
        Err(e) => {
            error!("❌ Login failed: {:?}", e);
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

    info!("\n=== Testing Historical Prices by Count Endpoint (API v1) ===");
    info!("Endpoint: /prices/{{epic}}/{{resolution}}/{{numPoints}}");
    info!("Description: Returns a specified number of historical data points (API v1)");

    // Example 1: Last 10 minutes of data
    info!("\n🔍 Example 1: Last 10 minutes of data");
    match market_service
        .get_historical_prices_by_count_v1(
            &session, &epic, "MINUTE", 10, // Last 10 minutes
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices (v1) obtained: {} data points",
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
                info!("  📈 Oldest price data:");
                info!("    Time: {}", first_price.snapshot_time);
                info!(
                    "    Close - Bid: {:?}, Ask: {:?}",
                    first_price.close_price.bid, first_price.close_price.ask
                );
            }

            if let Some(last_price) = prices.prices.last() {
                info!("  📈 Latest price data:");
                info!("    Time: {}", last_price.snapshot_time);
                info!(
                    "    Close - Bid: {:?}, Ask: {:?}",
                    last_price.close_price.bid, last_price.close_price.ask
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices (v1): {:?}", e);
        }
    }

    // Example 2: Last 24 hours of hourly data
    info!("\n🔍 Example 2: Last 24 hours of hourly data");
    match market_service
        .get_historical_prices_by_count_v1(
            &session, &epic, "HOUR", 24, // Last 24 hours
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Hourly prices (v1) obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }

            // Show first few and last few data points
            let show_count = std::cmp::min(3, prices.prices.len());

            info!("  📊 First {} hourly candles:", show_count);
            for (i, price) in prices.prices.iter().take(show_count).enumerate() {
                info!(
                    "    {}. {} - OHLC(Bid): O:{:?} H:{:?} L:{:?} C:{:?}",
                    i + 1,
                    price.snapshot_time,
                    price.open_price.bid,
                    price.high_price.bid,
                    price.low_price.bid,
                    price.close_price.bid
                );
            }

            if prices.prices.len() > show_count {
                info!("  📊 Last {} hourly candles:", show_count);
                for (i, price) in prices.prices.iter().rev().take(show_count).enumerate() {
                    info!(
                        "    {}. {} - OHLC(Bid): O:{:?} H:{:?} L:{:?} C:{:?}",
                        i + 1,
                        price.snapshot_time,
                        price.open_price.bid,
                        price.high_price.bid,
                        price.low_price.bid,
                        price.close_price.bid
                    );
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get hourly prices (v1): {:?}", e);
        }
    }

    // Example 3: Last 7 days of daily data
    info!("\n🔍 Example 3: Last 7 days of daily data");
    match market_service
        .get_historical_prices_by_count_v1(
            &session, &epic, "DAY", 7, // Last 7 days
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Daily prices (v1) obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }

            for (i, price) in prices.prices.iter().enumerate() {
                info!("  📅 Day {}: {}", i + 1, price.snapshot_time);
                info!(
                    "    Bid OHLC: Open:{:?} High:{:?} Low:{:?} Close:{:?}",
                    price.open_price.bid,
                    price.high_price.bid,
                    price.low_price.bid,
                    price.close_price.bid
                );
                info!(
                    "    Ask OHLC: Open:{:?} High:{:?} Low:{:?} Close:{:?}",
                    price.open_price.ask,
                    price.high_price.ask,
                    price.low_price.ask,
                    price.close_price.ask
                );
                if let Some(volume) = price.last_traded_volume {
                    info!("    Volume: {}", volume);
                }
                info!(""); // Empty line for readability
            }
        }
        Err(e) => {
            error!("❌ Failed to get daily prices (v1): {:?}", e);
        }
    }

    // Example 4: 5-minute resolution with 50 data points
    info!("\n🔍 Example 4: 5-minute resolution with 50 data points");
    match market_service
        .get_historical_prices_by_count_v1(
            &session, &epic, "MINUTE_5", 50, // Last 50 five-minute candles
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ 5-minute prices (v1) obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }

            if !prices.prices.is_empty() {
                info!(
                    "  📅 Time range: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );

                // Calculate some basic statistics
                let bid_closes: Vec<f64> = prices
                    .prices
                    .iter()
                    .filter_map(|p| p.close_price.bid)
                    .collect();

                if !bid_closes.is_empty() {
                    let min_price = bid_closes.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max_price = bid_closes.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    let avg_price = bid_closes.iter().sum::<f64>() / bid_closes.len() as f64;

                    info!("  📊 Price statistics (Bid Close):");
                    info!("    Min: {:.5}", min_price);
                    info!("    Max: {:.5}", max_price);
                    info!("    Avg: {:.5}", avg_price);
                    info!("    Range: {:.5}", max_price - min_price);
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get 5-minute prices (v1): {:?}", e);
        }
    }

    info!("\n=== Summary ===");
    info!("📋 Historical Prices by Count Endpoint (API v1) Features:");
    info!("  • Endpoint: /prices/{{epic}}/{{resolution}}/{{numPoints}}");
    info!("  • Purpose: Get exact number of historical data points");
    info!("  • Parameters: epic, resolution, number of points");
    info!("  • Supported resolutions: MINUTE, MINUTE_2, MINUTE_3, MINUTE_5,");
    info!("    MINUTE_10, MINUTE_15, MINUTE_30, HOUR, HOUR_2, HOUR_3, HOUR_4, DAY, WEEK, MONTH");
    info!("  • Returns: Specified number of most recent data points");
    info!("  • API Version: 1 (basic response format)");
    info!("  • Ideal for: Fixed-size datasets, technical analysis, charting");

    info!("\n=== Example completed successfully! ===");

    Ok(())
}
