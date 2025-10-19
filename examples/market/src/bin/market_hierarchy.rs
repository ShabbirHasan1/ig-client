use ig_client::application::models::market::MarketNode;
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::presentation::build_market_hierarchy;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::MarketService, config::Config, error::AppError, session::auth::IgAuth,
    session::interface::IgAuthenticator, transport::http_client::IgHttpClientImpl,
};
use std::{error::Error, sync::Arc};
use tracing::{error, info, warn};

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
    let session = auth
        .login()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Login successful");

    let session = match auth
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

    // First test with a simple request to verify the API
    info!("Testing API with a simple request...");
    match market_service.get_market_navigation(&session).await {
        Ok(response) => {
            info!(
                "Test successful: {} nodes, {} markets at top level",
                response.nodes.len(),
                response.markets.len()
            );

            // If the test is successful, build the complete hierarchy
            info!("Building market hierarchy...");
            let hierarchy = match build_market_hierarchy(&market_service, &session, None, 0).await {
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
