use ig_client::application::models::market::MarketNode;
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::presentation::build_market_hierarchy;
use ig_client::storage::market_database::MarketDatabaseService;
use ig_client::storage::utils::{create_connection_pool, create_database_config_from_env};
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::MarketService, config::Config, error::AppError, session::auth::IgAuth,
    session::interface::IgAuthenticator, transport::http_client::IgHttpClientImpl,
};
use std::{error::Error, sync::Arc};
use tokio;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure logger with more detail for debugging
    setup_logger();

    // Load configuration from environment variables
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));

    // Create HTTP client
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));

    // Create services
    let _account_service = AccountServiceImpl::new(config.clone(), client.clone());
    let market_service = MarketServiceImpl::new(config.clone(), client.clone());

    // Create authenticator
    let auth = IgAuth::new(&config);

    // Create database configuration and connection pool
    info!("Setting up database connection...");
    let db_config = create_database_config_from_env().map_err(|e| Box::new(e) as Box<dyn Error>)?;
    let pool = create_connection_pool(&db_config)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Database connection established");

    // Create database service
    let db_service = MarketDatabaseService::new(pool, "IG".to_string());

    // Initialize database tables
    info!("Initializing database tables...");
    db_service.initialize_database().await?;
    info!("Database tables initialized successfully");

    // Login to IG
    info!("Logging in to IG...");
    let session = auth
        .login()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Login successful");

    // Switch to the correct account if needed
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

    // Test API connectivity first
    info!("Testing API connectivity...");
    match market_service.get_market_navigation(&session).await {
        Ok(response) => {
            info!(
                "API test successful: {} nodes, {} markets at top level",
                response.nodes.len(),
                response.markets.len()
            );

            // Build the complete market hierarchy
            info!("Building complete market hierarchy...");
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
                    info!("Attempting to build a partial hierarchy...");

                    // Create a partial hierarchy with just the top-level nodes
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

                    warn!(
                        "Created partial hierarchy with {} top-level nodes due to API limitations",
                        limited_nodes.len()
                    );
                    limited_nodes
                }
            };

            // Store the hierarchy in the database
            info!("Storing market hierarchy in PostgreSQL database...");
            match db_service.store_market_hierarchy(&hierarchy).await {
                Ok(()) => {
                    info!("✅ Market hierarchy successfully stored in database");

                    // Get and display statistics
                    match db_service.get_statistics().await {
                        Ok(stats) => {
                            stats.print_summary();
                        }
                        Err(e) => {
                            warn!("Could not retrieve database statistics: {:?}", e);
                        }
                    }

                    // Demonstrate search functionality
                    info!("Testing search functionality...");
                    match db_service.search_instruments("Germany").await {
                        Ok(instruments) => {
                            info!(
                                "Found {} instruments matching 'Germany':",
                                instruments.len()
                            );
                            for (i, instrument) in instruments.iter().take(5).enumerate() {
                                info!(
                                    "  {}. {} ({})",
                                    i + 1,
                                    instrument.instrument_name,
                                    instrument.epic
                                );
                            }
                            if instruments.len() > 5 {
                                info!("  ... and {} more", instruments.len() - 5);
                            }
                        }
                        Err(e) => {
                            warn!("Search test failed: {:?}", e);
                        }
                    }

                    // Save hierarchy to JSON file as backup
                    let json = serde_json::to_string_pretty(&hierarchy)
                        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
                    let filename = "Data/market_hierarchy_backup.json";
                    std::fs::write(filename, &json).map_err(|e| Box::new(e) as Box<dyn Error>)?;
                    info!("Backup JSON file saved to '{}'", filename);
                }
                Err(e) => {
                    error!("Failed to store hierarchy in database: {:?}", e);

                    // Save to JSON file as fallback
                    let json = serde_json::to_string_pretty(&hierarchy)
                        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
                    let filename = "Data/market_hierarchy_fallback.json";
                    std::fs::write(filename, &json).map_err(|e| Box::new(e) as Box<dyn Error>)?;
                    info!("Fallback JSON file saved to '{}'", filename);

                    return Err(Box::new(e) as Box<dyn Error>);
                }
            }
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

    info!("Market hierarchy processing completed successfully!");
    Ok(())
}
