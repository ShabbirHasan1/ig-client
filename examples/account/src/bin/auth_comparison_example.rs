/// Example comparing API v2 (CST) and v3 (OAuth) authentication
///
/// This example demonstrates the differences between:
/// - API v2: CST/X-SECURITY-TOKEN authentication
/// - API v3: OAuth authentication
///
/// To run this example:
/// ```bash
/// cargo run --bin auth_comparison_example
/// ```
use ig_client::config::Config;
use ig_client::prelude::setup_logger;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::utils::rate_limiter::RateLimitType;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    setup_logger();
    info!("=== IG Markets Authentication Comparison ===\n");

    // Test API v2 (CST/X-SECURITY-TOKEN)
    info!("--- Testing API v2 (CST/X-SECURITY-TOKEN) ---");
    let mut config_v2 = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.8);
    config_v2.api_version = Some(2);

    let auth_v2 = IgAuth::new(&config_v2);
    match auth_v2.login().await {
        Ok(session) => {
            info!("✓ API v2 authentication successful");
            info!("  Account ID: {}", session.account_id);
            info!("  CST token length: {}", session.cst.len());
            info!("  X-SECURITY-TOKEN length: {}", session.token.len());
            info!("  Uses OAuth: {}", session.is_oauth());
            info!("  Uses CST auth: {}", session.is_cst_auth());
        }
        Err(e) => {
            error!("✗ API v2 authentication failed: {:?}", e);
        }
    }

    info!("\n--- Testing API v3 (OAuth) ---");
    let mut config_v3 = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.8);
    config_v3.api_version = Some(3);

    let auth_v3 = IgAuth::new(&config_v3);
    match auth_v3.login().await {
        Ok(session) => {
            info!("✓ API v3 authentication successful");
            info!("  Account ID: {}", session.account_id);
            info!("  Client ID: {}", session.client_id);
            info!(
                "  Lightstreamer endpoint: {}",
                session.lightstreamer_endpoint
            );
            info!("  Uses OAuth: {}", session.is_oauth());
            info!("  Uses CST auth: {}", session.is_cst_auth());

            if let Some(oauth) = &session.oauth_token {
                info!("  OAuth token type: {}", oauth.token_type);
                info!("  Token expires in: {} seconds", oauth.expires_in);
            }
        }
        Err(e) => {
            error!("✗ API v3 authentication failed: {:?}", e);
        }
    }

    info!("\n--- Testing Auto-detection (defaults to v3) ---");
    let mut config_auto = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.8);
    config_auto.api_version = None; // Will default to v3

    let auth_auto = IgAuth::new(&config_auto);
    match auth_auto.login().await {
        Ok(session) => {
            info!("✓ Auto-detection authentication successful");
            info!(
                "  Detected version: {}",
                if session.is_oauth() {
                    "v3 (OAuth)"
                } else {
                    "v2 (CST)"
                }
            );
            info!("  Account ID: {}", session.account_id);
        }
        Err(e) => {
            error!("✗ Auto-detection authentication failed: {:?}", e);
        }
    }

    info!("\n=== Comparison Complete ===");
}
