use chrono::{Duration, Utc};
use ig_client::application::services::MarketService;
use ig_client::storage::historical_prices::{
    get_table_statistics, initialize_historical_prices_table, store_historical_prices,
};
use ig_client::storage::utils::{create_connection_pool, create_database_config_from_env};
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

    info!("Configuration loaded:");
    info!("  Base URL: {}", config.rest_api.base_url);
    info!("  Username: {}", config.credentials.username);

    // Create database configuration
    let db_config = match create_database_config_from_env() {
        Ok(config) => {
            info!("✅ Database configuration loaded from environment");
            config
        }
        Err(e) => {
            error!("❌ Failed to load database configuration: {:?}", e);
            error!("Please set DATABASE_URL environment variable");
            return Err(e.into());
        }
    };

    // Create database connection pool
    let pool = match create_connection_pool(&db_config).await {
        Ok(pool) => {
            info!("✅ Database connection pool created successfully");
            pool
        }
        Err(e) => {
            error!("❌ Failed to create database connection pool: {:?}", e);
            return Err(e.into());
        }
    };

    // Initialize database table
    if let Err(e) = initialize_historical_prices_table(&pool).await {
        error!("❌ Failed to initialize database table: {:?}", e);
        return Err(e.into());
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
    let session = authenticator
        .login_and_switch_account(&config.credentials.account_id, Some(false))
        .await?;

    // Get epic from environment variable
    info!("\n=== Getting Market Epic from Environment ===");
    let epic = std::env::var("IG_EPIC").unwrap_or_else(|_| {
        warn!("⚠️  IG_EPIC environment variable not set, using default: CS.D.EURUSD.TODAY.IP");
        "CS.D.EURUSD.TODAY.IP".to_string()
    });

    info!("✅ Using epic: {}", epic);

    // Calculate date range for the last year
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(365);

    info!("\n=== Fetching Historical Data ===");
    info!("Epic: {}", epic);
    info!("Resolution: HOUR");
    info!(
        "Date range: {} to {}",
        start_date.format("%Y-%m-%d %H:%M:%S"),
        end_date.format("%Y-%m-%d %H:%M:%S")
    );

    // Fetch historical prices using the endpoint /prices/{epic}/{resolution}/{startDate}/{endDate} (API v2)
    match market_service
        .get_historical_prices_by_date_range(
            &session,
            &epic,
            "HOUR",
            &start_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            &end_date.format("%Y-%m-%d %H:%M:%S").to_string(),
        )
        .await
    {
        Ok(prices) => {
            info!(
                "✅ Historical prices obtained: {} data points",
                prices.prices.len()
            );

            if let Some(allowance) = &prices.allowance {
                info!("📊 API Allowance:");
                info!("  Remaining: {}", allowance.remaining_allowance);
                info!("  Total: {}", allowance.total_allowance);
                info!("  Expires in: {} seconds", allowance.allowance_expiry);
            }

            if !prices.prices.is_empty() {
                info!(
                    "📅 Data range: {} to {}",
                    prices.prices.first().unwrap().snapshot_time,
                    prices.prices.last().unwrap().snapshot_time
                );

                // Store data in PostgreSQL
                info!("\n=== Storing Data in PostgreSQL ===");
                match store_historical_prices(&pool, &epic, &prices.prices).await {
                    Ok(stats) => {
                        info!("✅ Data storage completed successfully!");
                        info!("📊 Storage Statistics:");
                        info!("  Records inserted: {}", stats.inserted);
                        info!("  Records updated: {}", stats.updated);
                        info!("  Records skipped: {}", stats.skipped);
                        info!("  Total processed: {}", stats.total_processed);
                    }
                    Err(e) => {
                        error!("❌ Failed to store data in database: {:?}", e);
                        return Err(e.into());
                    }
                }

                // Show some sample data
                info!("\n=== Sample Data ===");
                let sample_count = std::cmp::min(5, prices.prices.len());
                for (i, price) in prices.prices.iter().take(sample_count).enumerate() {
                    if let (Some(open), Some(high), Some(low), Some(close)) = (
                        price.open_price.bid,
                        price.high_price.bid,
                        price.low_price.bid,
                        price.close_price.bid,
                    ) {
                        info!(
                            "  {}. {} - OHLC: {:.5}/{:.5}/{:.5}/{:.5}",
                            i + 1,
                            price.snapshot_time,
                            open,
                            high,
                            low,
                            close
                        );
                    }
                }

                // Query and display database statistics
                info!("\n=== Database Statistics ===");
                match get_table_statistics(&pool, &epic).await {
                    Ok(stats) => {
                        info!("📊 Table: historical_prices");
                        info!("  Total records for {}: {}", epic, stats.total_records);
                        info!(
                            "  Date range: {} to {}",
                            stats.earliest_date, stats.latest_date
                        );
                        info!("  Average price: {:.5}", stats.avg_close_price);
                        info!("  Min price: {:.5}", stats.min_price);
                        info!("  Max price: {:.5}", stats.max_price);
                    }
                    Err(e) => {
                        warn!("⚠️  Could not retrieve table statistics: {:?}", e);
                    }
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to get historical prices: {:?}", e);
            return Err(e.into());
        }
    }
    

    info!("\n=== Example completed successfully! ===");

    Ok(())
}
