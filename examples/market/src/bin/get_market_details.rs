use ig_client::application::services::MarketService;
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::config::Config;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::transport::http_client::IgHttpClientImpl;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let epics = [
        "DO.D.OTCDDAX.151.IP",
        "DO.D.OTCDDAX.152.IP",
        "DO.D.OTCDDAX.153.IP",
        "DO.D.OTCDDAX.154.IP",
        "DO.D.OTCDDAX.155.IP",
        "DO.D.OTCDDAX.156.IP",
        "DO.D.OTCDDAX.157.IP",
        "DO.D.OTCDDAX.158.IP",
        "DO.D.OTCDDAX.159.IP",
        "DO.D.OTCDDAX.160.IP",
        "DO.D.OTCDDAX.161.IP",
        "DO.D.OTCDDAX.162.IP",
        "DO.D.OTCDDAX.163.IP",
        "DO.D.OTCDDAX.164.IP",
        "DO.D.OTCDDAX.165.IP",
        "DO.D.OTCDDAX.166.IP",
        "DO.D.OTCDDAX.167.IP",
        "DO.D.OTCDDAX.168.IP",
        "DO.D.OTCDDAX.169.IP",
        "DO.D.OTCDDAX.170.IP",
        "DO.D.OTCDDAX.171.IP",
        "DO.D.OTCDDAX.172.IP",
        "DO.D.OTCDDAX.173.IP",
        "DO.D.OTCDDAX.174.IP",
        "DO.D.OTCDDAX.175.IP",
    ];

    let mut config = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.7);
    // Use API v3 (OAuth) by default
    config.api_version = Some(3);
    let config = Arc::new(config);
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));
    let auth = IgAuth::new(&config);
    let market_service = MarketServiceImpl::new(config.clone(), client);
    let session = auth
        .login()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    let mut markets_got = Vec::new();

    for epic in epics {
        info!("Fetching market details for {} individually", epic);

        match market_service.get_market_details(&session, epic).await {
            Ok(details) => {
                debug!("✅ Successfully fetched details for {}", epic);
                markets_got.push(details);
            }
            Err(e) => {
                error!("❌ Failed to fetch details for {}: {:?}", epic, e);
            }
        }
    }
    info!("Fetched {} of {}", markets_got.len(), epics.len());
    Ok(())
}
