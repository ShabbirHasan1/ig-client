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
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));

    info!("=== IG Client Historical Prices by Count Example (API v2) ===");
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

    info!("\n=== Testing Historical Prices by Count Endpoint (API v2) ===");
    info!("Endpoint: /prices/{{epic}}/{{resolution}}/{{numPoints}}");
    info!("Description: Returns a specified number of historical data points (API v2 - Enhanced)");

    // Example 1: Last 15 minutes of data
    info!("\n🔍 Example 1: Last 15 minutes of data");
    match market_service
        .get_historical_prices_by_count_v2(
            &session, &epic, "MINUTE", 15, // Last 15 minutes
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices (v2) obtained: {} data points",
                prices.prices.len()
            );
            info!("  Instrument type: {:?}", prices.instrument_type);

            if let Some(allowance) = &prices.allowance {
                info!("  📊 API Allowance (Enhanced v2 format):");
                info!("    Remaining: {}", allowance.remaining_allowance);
                info!("    Total: {}", allowance.total_allowance);
                info!("    Expires in: {} seconds", allowance.allowance_expiry);

                let usage_percentage = (allowance.total_allowance - allowance.remaining_allowance)
                    as f64
                    / allowance.total_allowance as f64
                    * 100.0;
                info!("    Usage: {:.1}%", usage_percentage);
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
            error!("❌ Failed to get historical prices (v2): {:?}", e);
        }
    }

    // Example 2: Last 12 hours of 30-minute data
    info!("\n🔍 Example 2: Last 12 hours of 30-minute data");
    match market_service
        .get_historical_prices_by_count_v2(
            &session,
            &epic,
            "MINUTE_30",
            24, // 24 thirty-minute periods = 12 hours
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ 30-minute prices (v2) obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!(
                    "  📊 Remaining allowance: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }

            // Show detailed analysis of the data
            if !prices.prices.is_empty() {
                info!("  📊 30-minute candle analysis:");

                let mut total_volume = 0i64;
                let mut price_changes = Vec::new();

                for (i, price) in prices.prices.iter().enumerate() {
                    if let (Some(open_bid), Some(close_bid)) =
                        (price.open_price.bid, price.close_price.bid)
                    {
                        let change = close_bid - open_bid;
                        price_changes.push(change);

                        let change_pct = (change / open_bid) * 100.0;
                        let direction = if change > 0.0 {
                            "📈"
                        } else if change < 0.0 {
                            "📉"
                        } else {
                            "➡️"
                        };

                        info!(
                            "    {}. {} {} Change: {:+.5} ({:+.3}%)",
                            i + 1,
                            price.snapshot_time,
                            direction,
                            change,
                            change_pct
                        );

                        if let Some(volume) = price.last_traded_volume {
                            total_volume += volume;
                        }
                    }
                }

                if !price_changes.is_empty() {
                    let total_change: f64 = price_changes.iter().sum();
                    let avg_change = total_change / price_changes.len() as f64;
                    let positive_moves = price_changes.iter().filter(|&&x| x > 0.0).count();
                    let negative_moves = price_changes.iter().filter(|&&x| x < 0.0).count();

                    info!("  📊 Summary statistics:");
                    info!("    Total price change: {:+.5}", total_change);
                    info!("    Average change per period: {:+.5}", avg_change);
                    info!(
                        "    Positive moves: {} | Negative moves: {}",
                        positive_moves, negative_moves
                    );
                    if total_volume > 0 {
                        info!("    Total volume: {}", total_volume);
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get 30-minute prices (v2): {:?}", e);
        }
    }

    // Example 3: Last 5 days of daily data with detailed analysis
    info!("\n🔍 Example 3: Last 5 days of daily data with detailed analysis");
    match market_service
        .get_historical_prices_by_count_v2(
            &session, &epic, "DAY", 5, // Last 5 days
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Daily prices (v2) obtained: {} data points",
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

                // Calculate daily range and body size
                if let (Some(open_bid), Some(high_bid), Some(low_bid), Some(close_bid)) = (
                    price.open_price.bid,
                    price.high_price.bid,
                    price.low_price.bid,
                    price.close_price.bid,
                ) {
                    let daily_range = high_bid - low_bid;
                    let body_size = (close_bid - open_bid).abs();
                    let upper_shadow = high_bid - open_bid.max(close_bid);
                    let lower_shadow = open_bid.min(close_bid) - low_bid;

                    let candle_type = if close_bid > open_bid {
                        "🟢 Bullish"
                    } else if close_bid < open_bid {
                        "🔴 Bearish"
                    } else {
                        "⚪ Doji"
                    };

                    info!(
                        "    {} | OHLC: {:.5}/{:.5}/{:.5}/{:.5}",
                        candle_type, open_bid, high_bid, low_bid, close_bid
                    );
                    info!(
                        "    Range: {:.5} | Body: {:.5} | Shadows: {:.5}/{:.5}",
                        daily_range, body_size, upper_shadow, lower_shadow
                    );

                    if let Some(volume) = price.last_traded_volume {
                        info!("    Volume: {}", volume);
                    }
                }
                info!(""); // Empty line for readability
            }
        }
        Err(e) => {
            error!("❌ Failed to get daily prices (v2): {:?}", e);
        }
    }

    // Example 4: Comparison with different resolutions
    info!("\n🔍 Example 4: Comparison of different resolutions (last 6 data points each)");

    let resolutions = vec![
        ("MINUTE_5", "5-minute"),
        ("MINUTE_15", "15-minute"),
        ("HOUR", "1-hour"),
        ("HOUR_4", "4-hour"),
    ];

    for (resolution, description) in resolutions {
        info!("\n  📊 Testing {} resolution:", description);

        match market_service
            .get_historical_prices_by_count_v2(
                &session, &epic, resolution, 6, // Last 6 data points
            )
            .await
        {
            Ok(prices) => {
                info!(
                    "    ✅ {} data obtained: {} points",
                    description,
                    prices.prices.len()
                );

                if let Some(allowance) = &prices.allowance {
                    info!(
                        "    📊 Allowance: {}/{}",
                        allowance.remaining_allowance, allowance.total_allowance
                    );
                }

                if !prices.prices.is_empty() {
                    let first = prices.prices.first().unwrap();
                    let last = prices.prices.last().unwrap();

                    info!(
                        "    📅 Time span: {} to {}",
                        first.snapshot_time, last.snapshot_time
                    );

                    if let (Some(first_close), Some(last_close)) =
                        (first.close_price.bid, last.close_price.bid)
                    {
                        let total_change = last_close - first_close;
                        let change_pct = (total_change / first_close) * 100.0;

                        info!(
                            "    📈 Price change: {:+.5} ({:+.3}%)",
                            total_change, change_pct
                        );
                    }
                }
            }
            Err(e) => {
                error!("    ❌ Failed to get {} data: {:?}", description, e);
            }
        }
    }

    info!("\n=== Summary ===");
    info!("📋 Historical Prices by Count Endpoint (API v2) Features:");
    info!("  • Endpoint: /prices/{{epic}}/{{resolution}}/{{numPoints}}");
    info!("  • Purpose: Get exact number of historical data points (Enhanced v2)");
    info!("  • Parameters: epic, resolution, number of points");
    info!("  • Supported resolutions: MINUTE, MINUTE_2, MINUTE_3, MINUTE_5,");
    info!("    MINUTE_10, MINUTE_15, MINUTE_30, HOUR, HOUR_2, HOUR_3, HOUR_4, DAY, WEEK, MONTH");
    info!("  • Returns: Specified number of most recent data points");
    info!("  • API Version: 2 (enhanced response format with better metadata)");
    info!("  • Enhanced features: Improved timestamp format, better error handling");
    info!("  • Ideal for: Technical analysis, charting, algorithmic trading");

    info!("\n=== Differences from v1 ===");
    info!("  • Enhanced timestamp format (yyyy/MM/dd hh:mm:ss)");
    info!("  • Improved error handling and response structure");
    info!("  • Better metadata in allowance information");
    info!("  • More consistent data formatting");

    info!("\n=== Example completed successfully! ===");

    Ok(())
}
