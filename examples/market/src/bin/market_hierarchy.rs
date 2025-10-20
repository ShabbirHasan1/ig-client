use ig_client::model::utils::build_market_hierarchy;
use ig_client::prelude::*;
use std::error::Error;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure logger with more detail for debugging
    setup_logger();

    let client = Client::default();

    match client.get_market_navigation().await {
        Ok(response) => {
            info!(
                "Test successful: {} nodes, {} markets at top level",
                response.nodes.len(),
                response.markets.len()
            );

            // If the test is successful, build the complete hierarchy
            info!("Building market hierarchy...");
            info!("This may take several minutes due to rate limiting...");

            // Build hierarchy with periodic token refresh
            let hierarchy = match build_hierarchy_with_refresh(&client).await {
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
    market_service: &Client,
) -> Result<Vec<MarketNode>, AppError> {
    // Build hierarchy recursively with token refresh
    let result = build_market_hierarchy(market_service, None, 0).await;

    // If we got an OAuth token expired error, refresh and retry
    match result {
        Err(AppError::OAuthTokenExpired) => {
            info!("Token expired during hierarchy build - refreshing and retrying");
            build_market_hierarchy(market_service, None, 0).await
        }
        other => other,
    }
}
