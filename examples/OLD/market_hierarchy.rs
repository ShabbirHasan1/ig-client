use ig_client::application::models::market::MarketNode;
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::presentation::build_market_hierarchy;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::MarketService,
    config::Config,
    error::AppError,
    session::auth::IgAuth,
    session::interface::{IgAuthenticator, IgSession},
    transport::http_client::IgHttpClientImpl,
};
use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure logger with more detail for debugging
    setup_logger();

    // Load configuration from environment variables
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.3,
    ));

    // Create HTTP client
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));

    // Create services
    let _account_service = AccountServiceImpl::new(config.clone(), client.clone());
    let market_service = MarketServiceImpl::new(config.clone(), client.clone());

    // Create authenticator
    let auth = IgAuth::new(&config);

    // Login
    info!("Logging in...");
    let mut session = auth
        .login()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Login successful");

    // Check if using OAuth and log token info
    if session.is_oauth() {
        info!("✓ Using OAuth authentication (API v3)");
        if let Some(oauth_token) = &session.oauth_token {
            info!("  Token expires in: {} seconds", oauth_token.expires_in);
        }
    }

    session = match auth
        .switch_account(&session, &config.credentials.account_id, Some(true))
        .await
    {
        Ok(new_session) => {
            info!("✅ Switched to account: {}", new_session.account_id);
            new_session
        }
        Err(e) => {
            warn!(
                "Could not switch to account {}: {:?}. Attempting to re-authenticate.",
                config.credentials.account_id, e
            );

            match auth.login().await {
                Ok(new_session) => {
                    info!(
                        "Re-authentication successful. Using account: {}",
                        new_session.account_id
                    );
                    new_session
                }
                Err(login_err) => {
                    error!(
                        "Re-authentication failed: {:?}. Using original session.",
                        login_err
                    );
                    session
                }
            }
        }
    };

    // Wrap session in Arc<Mutex> for thread-safe token refresh
    let session = Arc::new(Mutex::new(session));

    // Start background task for periodic token refresh
    let session_clone = session.clone();
    let config_clone = config.clone();
    tokio::spawn(async move {
        let mut refresh_interval = interval(Duration::from_secs(1800)); // 30 minutes
        loop {
            refresh_interval.tick().await;
            debug!("Periodic token refresh check");

            let mut sess = session_clone.lock().await;
            if sess.needs_token_refresh(Some(300)) {
                let auth = IgAuth::new(&config_clone);
                match auth.refresh(&sess).await {
                    Ok(new_session) => {
                        *sess = new_session;
                        info!("✓ Token refreshed successfully (periodic check)");
                    }
                    Err(e) => {
                        warn!("Failed to refresh token: {:?}", e);
                    }
                }
            }
        }
    });

    // First test with a simple request to verify the API
    info!("Testing API with a simple request...");

    // Ensure token is valid before making the request
    {
        let mut sess = session.lock().await;
        if sess.needs_token_refresh(Some(300)) {
            info!("Token needs refresh - refreshing now");
            match auth.refresh(&sess).await {
                Ok(new_session) => {
                    *sess = new_session;
                    info!("✓ Token refreshed successfully");
                }
                Err(e) => {
                    warn!(
                        "Failed to refresh token: {:?}, attempting re-authentication",
                        e
                    );
                    match auth.login().await {
                        Ok(new_session) => {
                            *sess = new_session;
                            info!("✓ Re-authenticated successfully");
                        }
                        Err(e) => {
                            error!("Re-authentication failed: {:?}", e);
                        }
                    }
                }
            }
        }
    }

    let sess = session.lock().await;
    match market_service.get_market_navigation(&sess).await {
        Ok(response) => {
            info!(
                "Test successful: {} nodes, {} markets at top level",
                response.nodes.len(),
                response.markets.len()
            );

            // Release the lock before building hierarchy
            drop(sess);

            // If the test is successful, build the complete hierarchy
            info!("Building market hierarchy...");
            info!("This may take several minutes due to rate limiting...");

            // Build hierarchy with periodic token refresh
            let hierarchy =
                match build_hierarchy_with_refresh(&market_service, &session, &auth).await {
                    Ok(h) => {
                        info!(
                            "Successfully built hierarchy with {} top-level nodes",
                            h.len()
                        );
                        h
                    }
                    Err(e) => {
                        error!("Error building complete hierarchy: {:?}", e);
                        info!("Attempting to build a partial hierarchy with rate limiting...");
                        // Try again with a smaller scope
                        let limited_nodes = response
                            .nodes
                            .iter()
                            .map(|n| MarketNode {
                                id: n.id.clone(),
                                name: n.name.clone(),
                                children: Vec::new(),
                                markets: Vec::new(),
                            })
                            .collect::<Vec<_>>();
                        info!(
                            "Created partial hierarchy with {} top-level nodes",
                            limited_nodes.len()
                        );
                        limited_nodes
                    }
                };

            // Convert to JSON and save to a file
            let json = serde_json::to_string_pretty(&hierarchy)
                .map_err(|e| Box::new(e) as Box<dyn Error>)?;
            let filename = "Data/market_hierarchy.json";
            std::fs::write(filename, &json).map_err(|e| Box::new(e) as Box<dyn Error>)?;

            info!("Market hierarchy saved to '{}'", filename);
            info!("Hierarchy contains {} top-level nodes", hierarchy.len());
        }
        Err(e) => {
            error!("Error in initial API test: {:?}", e);

            // Get the underlying cause of the error if possible
            let mut current_error: &dyn Error = &e;
            while let Some(source) = current_error.source() {
                error!("Error cause: {}", source);
                current_error = source;

                // If it's a deserialization error, provide more information
                if source.to_string().contains("Decode") {
                    info!("Attempting to get raw response for analysis...");
                    error!("The API response structure does not match our model.");
                    error!("The API may have changed or there might be an authentication issue.");
                }
            }

            // If it's a rate limit error, provide specific guidance
            if matches!(e, AppError::RateLimitExceeded | AppError::Unexpected(_)) {
                error!("API rate limit exceeded or access denied.");
                info!("Consider implementing exponential backoff or reducing request frequency.");
                info!(
                    "The demo account has limited API access. Try again later or use a production account."
                );
            }

            return Err(Box::new(e) as Box<dyn Error>);
        }
    }

    Ok(())
}

/// Builds market hierarchy with automatic token refresh
async fn build_hierarchy_with_refresh(
    market_service: &MarketServiceImpl<IgHttpClientImpl>,
    session: &Arc<Mutex<IgSession>>,
    auth: &IgAuth<'_>,
) -> Result<Vec<MarketNode>, AppError> {
    // Refresh token before starting
    let mut sess = session.lock().await;
    if sess.needs_token_refresh(Some(300)) {
        info!("Token needs refresh before building hierarchy");
        match auth.refresh(&sess).await {
            Ok(new_session) => {
                *sess = new_session;
                info!("✓ Token refreshed successfully");
            }
            Err(e) => {
                warn!("Failed to refresh token: {:?}", e);
                warn!("Refresh token may be expired - attempting full re-authentication");

                // If refresh fails, the refresh token itself is likely expired
                // Need to do a full login
                match auth.login().await {
                    Ok(new_session) => {
                        *sess = new_session;
                        info!("✓ Re-authenticated successfully with new tokens");
                    }
                    Err(login_err) => {
                        error!("Re-authentication also failed: {:?}", login_err);
                        return Err(AppError::Unauthorized);
                    }
                }
            }
        }
    }
    drop(sess);

    // Build hierarchy recursively with token refresh
    let sess = session.lock().await;
    let result = build_market_hierarchy(market_service, &sess, None, 0).await;
    drop(sess);

    // If we got an OAuth token expired error, refresh and retry
    match result {
        Err(AppError::OAuthTokenExpired) => {
            info!("Token expired during hierarchy build - refreshing and retrying");

            let mut sess = session.lock().await;
            match auth.refresh(&sess).await {
                Ok(new_session) => {
                    *sess = new_session;
                    info!("✓ Token refreshed after expiration");
                }
                Err(e) => {
                    warn!("Failed to refresh token: {:?}", e);
                    warn!("Refresh token may be expired - attempting full re-authentication");

                    // If refresh fails, try full login
                    match auth.login().await {
                        Ok(new_session) => {
                            *sess = new_session;
                            info!("✓ Re-authenticated successfully with new tokens");
                        }
                        Err(login_err) => {
                            error!("Re-authentication also failed: {:?}", login_err);
                            return Err(AppError::Unauthorized);
                        }
                    }
                }
            }
            drop(sess);

            let sess = session.lock().await;
            build_market_hierarchy(market_service, &sess, None, 0).await
        }
        other => other,
    }
}
