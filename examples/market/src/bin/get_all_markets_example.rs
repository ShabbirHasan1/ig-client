use ig_client::application::services::MarketService;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::market_service::MarketServiceImpl, config::Config,
    session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl, utils::logger::setup_logger,
};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Create configuration using the default Config implementation
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.5,
    ));

    // Create HTTP client
    let http_client = Arc::new(IgHttpClientImpl::new(Arc::clone(&config)));
    let market_service = MarketServiceImpl::new(Arc::clone(&config), Arc::clone(&http_client));
    // Create authenticator
    let authenticator = IgAuth::new(&config);
    let session = authenticator
        .login_and_switch_account(&config.credentials.account_id, Some(false))
        .await?;

    // Get all markets using the new method
    info!("\n=== Getting All Markets from Hierarchy ===");

    match market_service.get_all_markets(&session).await {
        Ok(all_markets) => {
            info!(
                "✅ Found {} total markets across all levels",
                all_markets.len()
            );

            // Show some sample markets
            info!("\n📊 Sample of markets found:");
            for (i, market) in all_markets.iter().take(10).enumerate() {
                info!("  {}. {} ({})", i + 1, market.instrument_name, market.epic);
            }

            if all_markets.len() > 10 {
                info!("  ... and {} more markets", all_markets.len() - 10);
            }

            // Group by instrument type
            let mut type_counts = std::collections::HashMap::new();
            for market in &all_markets {
                let type_str = format!("{:?}", market.instrument_type);
                *type_counts.entry(type_str).or_insert(0) += 1;
            }

            info!("\n📈 Markets by instrument type:");
            for (instrument_type, count) in type_counts {
                info!("  {}: {}", instrument_type, count);
            }
        }
        Err(e) => {
            error!("❌ Failed to fetch all markets: {:?}", e);
            return Err(e.into());
        }
    }

    info!("\n=== Example completed successfully! ===");
    info!("💡 Use different max_levels values to balance between coverage and speed:");
    info!("   - None (default 5): Maximum coverage, slower");
    info!("   - Some(3): Good balance of coverage and speed");
    info!("   - Some(1): Minimal coverage, fastest");

    Ok(())
}
