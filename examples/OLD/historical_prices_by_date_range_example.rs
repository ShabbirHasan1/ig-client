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

    info!("=== IG Client Historical Prices by Date Range Example (API v2) ===");
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

    info!("\n=== Testing Historical Prices by Date Range Endpoint (API v2) ===");
    info!("Endpoint: /prices/{{epic}}/{{resolution}}/{{startDate}}/{{endDate}}");
    info!("Description: Returns historical prices for a specific date range using path parameters");
    info!("Date format: yyyy-MM-dd HH:mm:ss");

    // Example 1: One hour of 5-minute data
    info!("\n🔍 Example 1: One hour of 5-minute data (2024-01-15 10:00-11:00)");
    match market_service
        .get_historical_prices_by_date_range(
            &session,
            &epic,
            "MINUTE_5",
            "2024-01-15 10:00:00",
            "2024-01-15 11:00:00",
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices by date range obtained: {} data points",
                prices.prices.len()
            );
            info!("  Instrument type: {:?}", prices.instrument_type);

            if let Some(allowance) = &prices.allowance {
                info!("  📊 API Allowance:");
                info!("    Remaining: {}", allowance.remaining_allowance);
                info!("    Total: {}", allowance.total_allowance);
                info!("    Expires in: {} seconds", allowance.allowance_expiry);
            }

            if !prices.prices.is_empty() {
                info!(
                    "  📅 Actual time range: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );

                // Show first few 5-minute candles
                let show_count = std::cmp::min(5, prices.prices.len());
                info!("  📊 First {} 5-minute candles:", show_count);

                for (i, price) in prices.prices.iter().take(show_count).enumerate() {
                    if let (Some(open), Some(high), Some(low), Some(close)) = (
                        price.open_price.bid,
                        price.high_price.bid,
                        price.low_price.bid,
                        price.close_price.bid,
                    ) {
                        let change = close - open;
                        let direction = if change > 0.0 {
                            "📈"
                        } else if change < 0.0 {
                            "📉"
                        } else {
                            "➡️"
                        };

                        info!(
                            "    {}. {} {} OHLC: {:.5}/{:.5}/{:.5}/{:.5} (Δ{:+.5})",
                            i + 1,
                            price.snapshot_time,
                            direction,
                            open,
                            high,
                            low,
                            close,
                            change
                        );
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices by date range: {:?}", e);
            info!("  💡 Note: Historical data might not be available for the specified date range");
        }
    }

    // Example 2: One full trading day of hourly data
    info!("\n🔍 Example 2: One full trading day of hourly data (2024-01-10)");
    match market_service
        .get_historical_prices_by_date_range(
            &session,
            &epic,
            "HOUR",
            "2024-01-10 00:00:00",
            "2024-01-10 23:59:59",
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Full day hourly data obtained: {} data points",
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
                    "  📅 Trading day analysis for: {}",
                    prices
                        .prices
                        .first()
                        .unwrap()
                        .snapshot_time
                        .split(' ')
                        .next()
                        .unwrap_or("Unknown")
                );

                // Calculate daily statistics
                let bid_prices: Vec<f64> = prices
                    .prices
                    .iter()
                    .filter_map(|p| p.close_price.bid)
                    .collect();

                if !bid_prices.is_empty() {
                    let day_open = prices
                        .prices
                        .first()
                        .and_then(|p| p.open_price.bid)
                        .unwrap_or(0.0);
                    let day_close = prices
                        .prices
                        .last()
                        .and_then(|p| p.close_price.bid)
                        .unwrap_or(0.0);
                    let day_high = bid_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                    let day_low = bid_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));

                    let daily_change = day_close - day_open;
                    let daily_range = day_high - day_low;
                    let daily_change_pct = (daily_change / day_open) * 100.0;

                    info!("  📊 Daily Summary:");
                    info!("    Open: {:.5} | Close: {:.5}", day_open, day_close);
                    info!("    High: {:.5} | Low: {:.5}", day_high, day_low);
                    info!(
                        "    Change: {:+.5} ({:+.3}%)",
                        daily_change, daily_change_pct
                    );
                    info!("    Range: {:.5}", daily_range);

                    // Show hourly breakdown
                    info!("  🕐 Hourly breakdown (first 6 hours):");
                    for (i, price) in prices.prices.iter().take(6).enumerate() {
                        if let Some(close) = price.close_price.bid {
                            let hour_change = if i > 0 {
                                if let Some(prev_close) =
                                    prices.prices.get(i - 1).and_then(|p| p.close_price.bid)
                                {
                                    close - prev_close
                                } else {
                                    0.0
                                }
                            } else {
                                0.0
                            };

                            info!(
                                "    Hour {}: {} - Close: {:.5} (Δ{:+.5})",
                                i + 1,
                                price.snapshot_time,
                                close,
                                hour_change
                            );
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get full day data: {:?}", e);
        }
    }

    // Example 3: Weekly data with daily resolution
    info!("\n🔍 Example 3: One week of daily data (2024-01-08 to 2024-01-14)");
    match market_service
        .get_historical_prices_by_date_range(
            &session,
            &epic,
            "DAY",
            "2024-01-08 00:00:00",
            "2024-01-14 23:59:59",
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Weekly daily data obtained: {} data points",
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
                    "  📅 Week analysis: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );

                let mut weekly_volume = 0i64;
                let mut bullish_days = 0;
                let mut bearish_days = 0;

                for (i, price) in prices.prices.iter().enumerate() {
                    if let (Some(open), Some(high), Some(low), Some(close)) = (
                        price.open_price.bid,
                        price.high_price.bid,
                        price.low_price.bid,
                        price.close_price.bid,
                    ) {
                        let daily_change = close - open;
                        let daily_range = high - low;
                        let body_size = daily_change.abs();
                        let upper_shadow = high - open.max(close);
                        let lower_shadow = open.min(close) - low;

                        let candle_analysis = if daily_change > 0.0 {
                            bullish_days += 1;
                            "🟢 Bullish"
                        } else if daily_change < 0.0 {
                            bearish_days += 1;
                            "🔴 Bearish"
                        } else {
                            "⚪ Doji"
                        };

                        info!(
                            "  📅 Day {}: {} {}",
                            i + 1,
                            price.snapshot_time,
                            candle_analysis
                        );
                        info!("    OHLC: {:.5}/{:.5}/{:.5}/{:.5}", open, high, low, close);
                        info!(
                            "    Change: {:+.5} | Range: {:.5} | Body: {:.5}",
                            daily_change, daily_range, body_size
                        );
                        info!(
                            "    Shadows: Upper {:.5} | Lower {:.5}",
                            upper_shadow, lower_shadow
                        );

                        if let Some(volume) = price.last_traded_volume {
                            weekly_volume += volume;
                            info!("    Volume: {}", volume);
                        }
                        info!(""); // Empty line for readability
                    }
                }

                info!("  📊 Weekly Summary:");
                info!(
                    "    Bullish days: {} | Bearish days: {}",
                    bullish_days, bearish_days
                );
                if weekly_volume > 0 {
                    info!("    Total volume: {}", weekly_volume);
                }

                if let (Some(week_open), Some(week_close)) = (
                    prices.prices.first().and_then(|p| p.open_price.bid),
                    prices.prices.last().and_then(|p| p.close_price.bid),
                ) {
                    let weekly_change = week_close - week_open;
                    let weekly_change_pct = (weekly_change / week_open) * 100.0;
                    info!(
                        "    Weekly change: {:+.5} ({:+.3}%)",
                        weekly_change, weekly_change_pct
                    );
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get weekly data: {:?}", e);
        }
    }

    // Example 4: Precise time window (market session)
    info!("\n🔍 Example 4: Precise market session (2024-01-12 08:00-16:00 London time)");
    match market_service
        .get_historical_prices_by_date_range(
            &session,
            &epic,
            "MINUTE_30",
            "2024-01-12 08:00:00",
            "2024-01-12 16:00:00",
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Market session data obtained: {} data points",
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
                    "  📅 Session analysis: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );

                // Analyze market session activity
                let session_data: Vec<_> = prices
                    .prices
                    .iter()
                    .filter_map(|p| {
                        if let (Some(open), Some(close), Some(high), Some(low)) = (
                            p.open_price.bid,
                            p.close_price.bid,
                            p.high_price.bid,
                            p.low_price.bid,
                        ) {
                            Some((p.snapshot_time.clone(), open, high, low, close))
                        } else {
                            None
                        }
                    })
                    .collect();

                if !session_data.is_empty() {
                    let session_open = session_data.first().unwrap().1;
                    let session_close = session_data.last().unwrap().4;
                    let session_high = session_data
                        .iter()
                        .map(|(_, _, h, _, _)| *h)
                        .fold(f64::NEG_INFINITY, f64::max);
                    let session_low = session_data
                        .iter()
                        .map(|(_, _, _, l, _)| *l)
                        .fold(f64::INFINITY, f64::min);

                    let session_change = session_close - session_open;
                    let session_range = session_high - session_low;

                    info!("  📊 Session Summary:");
                    info!(
                        "    Open: {:.5} | Close: {:.5}",
                        session_open, session_close
                    );
                    info!("    High: {:.5} | Low: {:.5}", session_high, session_low);
                    info!(
                        "    Change: {:+.5} | Range: {:.5}",
                        session_change, session_range
                    );

                    // Find most volatile 30-minute period
                    let mut max_volatility = 0.0;
                    let mut most_volatile_time = String::new();

                    for (time, _, high, low, _) in &session_data {
                        let volatility = high - low;
                        if volatility > max_volatility {
                            max_volatility = volatility;
                            most_volatile_time = time.clone();
                        }
                    }

                    info!(
                        "    Most volatile 30min: {} (range: {:.5})",
                        most_volatile_time, max_volatility
                    );
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get market session data: {:?}", e);
        }
    }

    info!("\n=== Summary ===");
    info!("📋 Historical Prices by Date Range Endpoint (API v2) Features:");
    info!("  • Endpoint: /prices/{{epic}}/{{resolution}}/{{startDate}}/{{endDate}}");
    info!("  • Purpose: Get historical data for specific date ranges");
    info!("  • Date format: yyyy-MM-dd HH:mm:ss (path parameters)");
    info!("  • Parameters: epic, resolution, start_date, end_date");
    info!("  • Supported resolutions: SECOND, MINUTE, MINUTE_2, MINUTE_3, MINUTE_5,");
    info!("    MINUTE_10, MINUTE_15, MINUTE_30, HOUR, HOUR_2, HOUR_3, HOUR_4, DAY, WEEK, MONTH");
    info!("  • Returns: All data points within the specified date range");
    info!("  • API Version: 2 (enhanced response format)");
    info!("  • Ideal for: Backtesting, historical analysis, specific time period studies");

    info!("\n=== Usage Tips ===");
    info!("  • Use precise time ranges for focused analysis");
    info!("  • Consider market hours when selecting time ranges");
    info!("  • Higher resolutions (SECOND, MINUTE) consume more allowance");
    info!("  • Date ranges must not be too large to avoid timeout");
    info!("  • End date must be later than start date");

    info!("\n=== Example completed successfully! ===");

    Ok(())
}
