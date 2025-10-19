/// Example demonstrating OAuth token refresh for IG Markets API
///
/// This example shows how to:
/// 1. Authenticate using OAuth (API v3)
/// 2. Check if tokens need refresh
/// 3. Handle OAuthTokenExpired errors
/// 4. Automatically refresh tokens when they expire
///
/// Run with: cargo run --example oauth_token_refresh_example
use dotenv::dotenv;
use ig_client::application::services::AccountService;
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::config::Config;
use ig_client::error::AppError;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::{IgAuthenticator, IgSession};
use ig_client::transport::http_client::IgHttpClientImpl;
use std::sync::Arc;
use tracing::{Level, debug, error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load environment variables
    dotenv().ok();
    info!("Environment variables loaded");

    // Create configuration - ensure API version is set to 3 for OAuth
    let mut config = Config::new();
    config.api_version = Some(3); // Force OAuth authentication
    let config = Arc::new(config);
    info!("Configuration created with OAuth (API v3)");

    // Authenticate with OAuth
    let auth = IgAuth::new(&config);
    let mut session = auth.login().await?;
    info!("Successfully authenticated with OAuth");

    // Verify we're using OAuth
    if session.is_oauth() {
        info!("✓ Session is using OAuth authentication");
        if let Some(oauth_token) = &session.oauth_token {
            info!("  Token expires in: {} seconds", oauth_token.expires_in);
        }
    } else {
        error!("✗ Session is not using OAuth - check API version configuration");
        return Ok(());
    }

    // Create HTTP client and account service
    let http_client = Arc::new(IgHttpClientImpl::new(config.clone()));
    let account_service = AccountServiceImpl::new(config.clone(), http_client);

    // Simulate making API calls with automatic token refresh
    for i in 1..=5 {
        info!("\n--- API Call {} ---", i);

        // Check if token needs refresh (with 5 minute safety margin)
        if session.needs_token_refresh(Some(300)) {
            info!("Token needs refresh - refreshing now...");
            match auth.refresh(&session).await {
                Ok(new_session) => {
                    session = new_session;
                    info!("✓ Token refreshed successfully");
                }
                Err(e) => {
                    error!("✗ Failed to refresh token: {:?}", e);
                    error!("Attempting full re-authentication...");
                    session = auth.login().await?;
                    info!("✓ Re-authenticated successfully");
                }
            }
        }

        // Make API call with error handling for expired tokens
        match make_api_call_with_retry(&account_service, &session, &auth).await {
            Ok(account_count) => {
                info!("✓ API call successful - found {} accounts", account_count);
            }
            Err(e) => {
                error!("✗ API call failed after retries: {:?}", e);
            }
        }

        // Wait a bit between calls (in real usage, this would be your normal operation)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    info!("\nExample completed successfully");
    Ok(())
}

/// Makes an API call with automatic token refresh on expiration
///
/// This function demonstrates the recommended pattern for handling OAuth token expiration:
/// 1. Try the API call
/// 2. If it fails with OAuthTokenExpired, refresh the token
/// 3. Retry the API call with the new token
async fn make_api_call_with_retry(
    account_service: &AccountServiceImpl<IgHttpClientImpl>,
    session: &IgSession,
    auth: &IgAuth<'_>,
) -> Result<usize, AppError> {
    // First attempt
    match account_service.get_accounts(session).await {
        Ok(accounts) => {
            debug!("API call succeeded on first attempt");
            Ok(accounts.accounts.len())
        }
        Err(AppError::OAuthTokenExpired) => {
            info!("OAuth token expired - attempting refresh...");

            // Refresh the token
            let new_session = auth
                .refresh(session)
                .await
                .map_err(|_e| AppError::Unauthorized)?;

            info!("Token refreshed - retrying API call...");

            // Retry with new token
            match account_service.get_accounts(&new_session).await {
                Ok(accounts) => {
                    debug!("API call succeeded after token refresh");
                    Ok(accounts.accounts.len())
                }
                Err(e) => {
                    error!("API call failed even after token refresh: {:?}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("API call failed with non-token error: {:?}", e);
            Err(e)
        }
    }
}
