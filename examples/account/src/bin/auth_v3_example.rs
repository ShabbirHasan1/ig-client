/// Example demonstrating OAuth authentication (API v3) with IG Markets
///
/// This example shows how to:
/// 1. Configure the client to use API v3 (OAuth)
/// 2. Authenticate and obtain OAuth tokens
/// 3. Use the session to make API calls
///
/// To run this example:
/// ```bash
/// cargo run --bin auth_v3_example
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
    info!("Starting OAuth (v3) authentication example");

    // Create configuration with API v3 enabled
    let mut config = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.8);
    config.api_version = Some(3); // Use OAuth authentication

    info!(
        "Configuration loaded with API version: {:?}",
        config.api_version
    );

    // Create authenticator
    let auth = IgAuth::new(&config);

    // Perform login using OAuth (v3)
    info!("Attempting login with OAuth (API v3)...");
    match auth.login().await {
        Ok(session) => {
            info!("✓ Successfully authenticated with OAuth!");
            info!("  Account ID: {}", session.account_id);
            info!("  Client ID: {}", session.client_id);
            info!(
                "  Lightstreamer endpoint: {}",
                session.lightstreamer_endpoint
            );

            // Check if OAuth tokens are present
            if let Some(oauth_token) = &session.oauth_token {
                info!("  OAuth token type: {}", oauth_token.token_type);
                info!("  Token scope: {}", oauth_token.scope);
                info!("  Token expires in: {} seconds", oauth_token.expires_in);
                info!("  Access token length: {}", oauth_token.access_token.len());
                info!(
                    "  Refresh token length: {}",
                    oauth_token.refresh_token.len()
                );
            } else {
                error!("  Warning: No OAuth tokens found in session!");
            }

            // Verify session type
            if session.is_oauth() {
                info!("✓ Session is using OAuth authentication");
            } else if session.is_cst_auth() {
                info!("  Session is using CST/X-SECURITY-TOKEN authentication");
            }
        }
        Err(e) => {
            error!("✗ Authentication failed: {:?}", e);
            std::process::exit(1);
        }
    }

    info!("OAuth authentication example completed");
}
