use ig_client::application::services::AccountService;
/// Example demonstrating OAuth token refresh flow
///
/// This example shows:
/// 1. Login with OAuth (API v3)
/// 2. Wait for the access token to expire
/// 3. Refresh the token using the refresh token
/// 4. Get positions with the new token
///
/// OAuth tokens from IG typically expire in 60 seconds, and this example
/// demonstrates how to handle token expiration and refresh automatically.
///
/// To run this example:
/// ```bash
/// cargo run --bin oauth_token_refresh
/// ```
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::config::Config;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::transport::http_client::IgHttpClientImpl;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    setup_logger();

    info!("=== OAuth Token Refresh Example ===\n");

    // Create configuration with API v3 (OAuth)
    let mut config = Config::with_rate_limit_type(RateLimitType::NonTradingAccount, 0.7);
    config.api_version = Some(3);
    let config = Arc::new(config);

    info!("Configuration loaded:");
    info!("  Base URL: {}", config.rest_api.base_url);
    info!("  API Version: v3 (OAuth)");

    // Create HTTP client and services
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));
    let account_service = AccountServiceImpl::new(config.clone(), client);
    let auth = IgAuth::new(&config);

    // Step 1: Initial login with OAuth
    info!("\n1. Logging in with OAuth (API v3)...");
    let mut session = match auth.login().await {
        Ok(s) => s,
        Err(e) => {
            error!("✗ Login failed: {:?}", e);
            return Err(format!("Login error: {:?}", e).into());
        }
    };

    info!("✓ Login successful");
    info!("  Account ID: {}", session.account_id);
    info!("  Client ID: {}", session.client_id);

    if let Some(oauth) = &session.oauth_token {
        info!("  OAuth Token Info:");
        info!(
            "    Access token (first 10 chars): {}...",
            &oauth.access_token[..10.min(oauth.access_token.len())]
        );
        info!(
            "    Refresh token (first 10 chars): {}...",
            &oauth.refresh_token[..10.min(oauth.refresh_token.len())]
        );
        info!("    Token type: {}", oauth.token_type);
        info!("    Expires in: {} seconds", oauth.expires_in);
        info!("    Scope: {}", oauth.scope);
    } else {
        error!("✗ No OAuth token received!");
        return Err("Expected OAuth token but got none".into());
    }

    // Step 2: Get positions with initial token
    info!("\n2. Getting positions with initial token...");
    match account_service.get_positions(&session).await {
        Ok(positions) => {
            info!("✓ Successfully retrieved positions");
            info!("  Total positions: {}", positions.positions.len());

            if !positions.positions.is_empty() {
                info!("\n  Sample positions:");
                for (i, position) in positions.positions.iter().take(3).enumerate() {
                    info!(
                        "  {}. {} - {} @ {} (Size: {})",
                        i + 1,
                        position.market.instrument_name,
                        position.market.epic,
                        position.position.direction,
                        position.position.size
                    );
                }
                if positions.positions.len() > 3 {
                    info!("  ... and {} more", positions.positions.len() - 3);
                }
            } else {
                info!("  No open positions");
            }
        }
        Err(e) => {
            error!("✗ Failed to get positions: {:?}", e);
            return Err(format!("Get positions error: {:?}", e).into());
        }
    }

    // Step 3: Wait for token to expire
    let expires_in = session
        .oauth_token
        .as_ref()
        .and_then(|t| t.expires_in.parse::<u64>().ok())
        .unwrap_or(60);

    info!("\n3. Waiting for access token to expire...");
    info!("  Token expires in {} seconds", expires_in);
    info!("  Waiting {} seconds + 5 second buffer...", expires_in);

    // Wait for token to expire plus a small buffer
    let wait_time = expires_in + 5;
    for i in (1..=wait_time).rev() {
        if i % 10 == 0 || i <= 5 {
            info!("  {} seconds remaining...", i);
        }
        sleep(Duration::from_secs(1)).await;
    }

    info!("✓ Token should now be expired");

    // Step 4: Try to use expired token (should fail)
    info!("\n4. Attempting to use expired token...");
    match account_service.get_positions(&session).await {
        Ok(_) => {
            warn!("⚠ Request succeeded with expired token (token might not have expired yet)");
        }
        Err(e) => {
            info!("✓ Request failed as expected: {:?}", e);
            info!("  This confirms the token has expired");
        }
    }

    // Step 5: Refresh the token (or re-login if refresh not available)
    info!("\n5. Refreshing OAuth token...");

    if let Some(oauth) = &session.oauth_token {
        let refresh_token = oauth.refresh_token.clone();
        info!(
            "  Using refresh token: {}...",
            &refresh_token[..10.min(refresh_token.len())]
        );

        match auth.refresh(&session).await {
            Ok(new_session) => {
                session = new_session;
                info!("✓ Token refresh successful");

                if let Some(new_oauth) = &session.oauth_token {
                    info!("  New OAuth Token Info:");
                    info!(
                        "    New access token (first 10 chars): {}...",
                        &new_oauth.access_token[..10.min(new_oauth.access_token.len())]
                    );
                    info!(
                        "    New refresh token (first 10 chars): {}...",
                        &new_oauth.refresh_token[..10.min(new_oauth.refresh_token.len())]
                    );
                    info!("    Expires in: {} seconds", new_oauth.expires_in);
                }
            }
            Err(e) => {
                warn!("⚠ Token refresh failed: {:?}", e);
                warn!("  The refresh endpoint might not be available on demo API");
                warn!("  Falling back to re-login...");

                // Fallback: re-login to get a new token
                match auth.login().await {
                    Ok(new_session) => {
                        session = new_session;
                        info!("✓ Re-login successful (workaround for missing refresh endpoint)");

                        if let Some(new_oauth) = &session.oauth_token {
                            info!("  New OAuth Token Info:");
                            info!(
                                "    New access token (first 10 chars): {}...",
                                &new_oauth.access_token[..10.min(new_oauth.access_token.len())]
                            );
                            info!("    Expires in: {} seconds", new_oauth.expires_in);
                        }
                    }
                    Err(e) => {
                        error!("✗ Re-login also failed: {:?}", e);
                        return Err(format!("Re-login error: {:?}", e).into());
                    }
                }
            }
        }
    } else {
        error!("✗ No OAuth token available for refresh");
        return Err("No OAuth token to refresh".into());
    }

    // Step 6: Get positions with refreshed token
    info!("\n6. Getting positions with refreshed token...");
    match account_service.get_positions(&session).await {
        Ok(positions) => {
            info!("✓ Successfully retrieved positions with new token");
            info!("  Total positions: {}", positions.positions.len());

            if !positions.positions.is_empty() {
                info!("\n  Current positions:");
                for (i, position) in positions.positions.iter().enumerate() {
                    info!(
                        "  {}. {} - {} @ {} (Size: {})",
                        i + 1,
                        position.market.instrument_name,
                        position.market.epic,
                        position.position.direction,
                        position.position.size
                    );
                }
            } else {
                info!("  No open positions");
            }
        }
        Err(e) => {
            error!("✗ Failed to get positions with refreshed token: {:?}", e);
            return Err(format!("Get positions error: {:?}", e).into());
        }
    }

    info!("\n=== Example Complete ===");
    info!("\nSummary:");
    info!("  ✓ Initial login with OAuth");
    info!("  ✓ Retrieved positions with initial token");
    info!("  ✓ Waited for token expiration");
    info!("  ✓ Obtained new token (via refresh or re-login)");
    info!("  ✓ Retrieved positions with new token");
    info!("\nNote: The demo API may not support the refresh endpoint,");
    info!("      so we fall back to re-login as a workaround.");

    Ok(())
}
