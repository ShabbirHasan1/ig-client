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

    info!("=== IG Client Historical Prices Endpoints Example ===");
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

    info!("\n=== Testing All Historical Prices Endpoints ===");

    // 1. Test /prices/{epic} (API v3) - Recent prices with defaults
    info!("\n🔍 1. Testing get_recent_prices (API v3) - Default parameters");
    let params = RecentPricesParams::new(&epic);
    match market_service.get_recent_prices(&session, &params).await {
        Ok(prices) => {
            info!(
                "✅ Recent prices obtained: {} data points",
                prices.prices.len()
            );
            if let Some(allowance) = &prices.allowance {
                info!(
                    "  Allowance remaining: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
            if let Some(first_price) = prices.prices.first() {
                info!("  First price snapshot: {}", first_price.snapshot_time);
                info!(
                    "  Close price - Bid: {:?}, Ask: {:?}",
                    first_price.close_price.bid, first_price.close_price.ask
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get recent prices: {:?}", e);
        }
    }

    // 2. Test /prices/{epic} (API v3) - Recent prices with custom parameters
    info!("\n🔍 2. Testing get_recent_prices (API v3) - Custom parameters");
    let params = RecentPricesParams::new(&epic)
        .with_resolution("MINUTE_5")
        .with_max_points(5)
        .with_page_size(10)
        .with_page_number(1);
    match market_service.get_recent_prices(&session, &params).await {
        Ok(prices) => {
            info!(
                "✅ Recent prices (5min) obtained: {} data points",
                prices.prices.len()
            );
            if let Some(allowance) = &prices.allowance {
                info!(
                    "  Allowance remaining: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get recent prices (5min): {:?}", e);
        }
    }

    // 3. Test /prices/{epic}/{resolution}/{numPoints} (API v1)
    info!("\n🔍 3. Testing get_historical_prices_by_count_v1 (API v1)");
    match market_service
        .get_historical_prices_by_count_v1(
            &session, &epic, "HOUR", 24, // Last 24 hours
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices (v1) obtained: {} data points",
                prices.prices.len()
            );
            if let Some(allowance) = &prices.allowance {
                info!(
                    "  Allowance remaining: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
            if let Some(first_price) = prices.prices.first() {
                info!("  First hourly price: {}", first_price.snapshot_time);
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices (v1): {:?}", e);
        }
    }

    // 4. Test /prices/{epic}/{resolution}/{numPoints} (API v2)
    info!("\n🔍 4. Testing get_historical_prices_by_count_v2 (API v2)");
    match market_service
        .get_historical_prices_by_count_v2(
            &session, &epic, "DAY", 7, // Last 7 days
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices (v2) obtained: {} data points",
                prices.prices.len()
            );
            if let Some(allowance) = &prices.allowance {
                info!(
                    "  Allowance remaining: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
            if let Some(first_price) = prices.prices.first() {
                info!("  First daily price: {}", first_price.snapshot_time);
                info!(
                    "  OHLC - Open: {:?}, High: {:?}, Low: {:?}, Close: {:?}",
                    first_price.open_price.bid,
                    first_price.high_price.bid,
                    first_price.low_price.bid,
                    first_price.close_price.bid
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices (v2): {:?}", e);
        }
    }

    // 5. Test /prices/{epic}/{resolution}/{startDate}/{endDate} (API v2)
    info!("\n🔍 5. Testing get_historical_prices_by_date_range (API v2)");
    match market_service
        .get_historical_prices_by_date_range(
            &session,
            &epic,
            "HOUR",
            "2024-01-01 00:00:00",
            "2024-01-01 23:59:59",
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices by date range obtained: {} data points",
                prices.prices.len()
            );
            if let Some(allowance) = &prices.allowance {
                info!(
                    "  Allowance remaining: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
            if !prices.prices.is_empty() {
                info!(
                    "  Date range: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices by date range: {:?}", e);
        }
    }

    // 6. Test original method /prices/{epic}/{resolution}?startdate=&enddate= (API v1)
    info!("\n🔍 6. Testing get_historical_prices (original method with query params)");
    match market_service
        .get_historical_prices(
            &session,
            &epic,
            "MINUTE_15",
            "2024-01-01T10:00:00",
            "2024-01-01T11:00:00",
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices (query params) obtained: {} data points",
                prices.prices.len()
            );
            if let Some(allowance) = &prices.allowance {
                info!(
                    "  Allowance remaining: {}/{}",
                    allowance.remaining_allowance, allowance.total_allowance
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices (query params): {:?}", e);
        }
    }

    info!("\n=== Summary of Available Historical Prices Methods ===");
    info!("1. get_recent_prices() - /prices/{{epic}} (API v3)");
    info!("   • Returns recent minute prices (last 10 minutes by default)");
    info!("   • Supports optional parameters: resolution, from, to, max, pageSize, pageNumber");
    info!("");
    info!(
        "2. get_historical_prices_by_count_v1() - /prices/{{epic}}/{{resolution}}/{{numPoints}} (API v1)"
    );
    info!("   • Returns specified number of historical data points");
    info!("   • Version 1 of the API");
    info!("");
    info!(
        "3. get_historical_prices_by_count_v2() - /prices/{{epic}}/{{resolution}}/{{numPoints}} (API v2)"
    );
    info!("   • Returns specified number of historical data points");
    info!("   • Version 2 of the API (enhanced response format)");
    info!("");
    info!(
        "4. get_historical_prices_by_date_range() - /prices/{{epic}}/{{resolution}}/{{startDate}}/{{endDate}} (API v2)"
    );
    info!("   • Returns historical prices for specific date range");
    info!("   • Uses path parameters for dates");
    info!("");
    info!(
        "5. get_historical_prices() - /prices/{{epic}}/{{resolution}}?startdate=&enddate= (API v1)"
    );
    info!("   • Returns historical prices for date range using query parameters");
    info!("   • Original implementation");

    info!("\n=== Example completed successfully! ===");
    info!("💡 All 4 IG Markets historical prices endpoints are now implemented:");
    info!("   • /prices/{{epic}} (v3) - Recent prices with flexible parameters");
    info!("   • /prices/{{epic}}/{{resolution}}/{{numPoints}} (v1 & v2) - By data point count");
    info!(
        "   • /prices/{{epic}}/{{resolution}}/{{startDate}}/{{endDate}} (v2) - By date range (path)"
    );
    info!("   • /prices/{{epic}}/{{resolution}}?startdate=&enddate= (v1) - By date range (query)");

    Ok(())
}
