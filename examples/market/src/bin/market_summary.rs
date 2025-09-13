use ig_client::application::services::MarketService;
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::config::Config;
use ig_client::presentation::{build_market_hierarchy, extract_markets_from_hierarchy};
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::transport::http_client::IgHttpClientImpl;
use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use std::error::Error;
use std::fs;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

// Constants for API request handling
const BATCH_SIZE: usize = 10; // Number of EPICs to process before saving results

/// Structure to hold market summary information
#[derive(Debug, Clone)]
struct MarketSummary {
    instrument_name: String,
    epic: String,
    bid: Option<f64>,
    offer: Option<f64>,
    mid: Option<f64>,
    spread: Option<f64>,
    last_dealing_date: String,
}

/// Fetches market details and returns a vector of MarketSummary structs
async fn get_market_summaries() -> Result<Vec<MarketSummary>, Box<dyn Error>> {
    // Create a directory for the output file if it doesn't exist
    std::fs::create_dir_all("Data").map_err(|e| Box::new(e) as Box<dyn Error>)?;

    // Configure with rate limiting for better API handling
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.8,
    ));
    info!("Loaded configuration → {}", config.rest_api.base_url);

    // Create the HTTP client
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));

    // Create the authenticator
    let auth = IgAuth::new(&config);

    // Create market service
    let market_service = MarketServiceImpl::new(config.clone(), client);

    // Login to get a session
    info!("Logging in...");
    let session = auth
        .login()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Login successful. Account ID: {}", session.account_id);

    // Use a very limited approach to get market data
    info!("Getting top-level market navigation nodes only...");
    let top_level = market_service
        .get_market_navigation(&session)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;

    info!(
        "Found {} top-level nodes and {} markets",
        top_level.nodes.len(),
        top_level.markets.len()
    );

    // Create a simple hierarchy with just the top-level data
    let mut markets = top_level.markets.clone();

    // If we have very few markets at the top level, try to get one more level
    // but only from the first node to avoid rate limits
    if markets.len() < 5 && !top_level.nodes.is_empty() {
        info!(
            "Getting markets from the first top-level node: {}",
            top_level.nodes[0].name
        );

        match market_service
            .get_market_navigation_node(&session, &top_level.nodes[0].id)
            .await
        {
            Ok(node_data) => {
                info!(
                    "Found {} additional markets in node {}",
                    node_data.markets.len(),
                    top_level.nodes[0].name
                );
                markets.extend(node_data.markets);
            }
            Err(e) => {
                warn!(
                    "Could not get markets from node {}: {:?}",
                    top_level.nodes[0].name, e
                );
            }
        }
    }

    // If no markets were found, return an empty result
    if markets.is_empty() {
        warn!("No markets found. Check API access or try again later.");
        return Ok(Vec::new());
    }

    // Take a very small sample of markets to process (up to 10)
    let sample_size = std::cmp::min(10, markets.len());
    let sample_markets = markets.into_iter().take(sample_size).collect::<Vec<_>>();

    // Extract EPICs from the market data
    let epics = sample_markets
        .iter()
        .map(|market| market.epic.clone())
        .collect::<Vec<String>>();

    info!(
        "Selected {} EPICs for detailed market information",
        epics.len()
    );

    info!("Fetching market details for {} EPICs", epics.len());

    // Create a vector to store all the market details
    let mut market_summaries = Vec::new();
    // Process EPICs in batches to use the API more efficiently
    let mut processed_count = 0;
    let total_epics = epics.len();

    // Save the list of EPICs to a file for reference
    let epics_json =
        serde_json::to_string_pretty(&epics).map_err(|e| Box::new(e) as Box<dyn Error>)?;
    fs::write("Data/market_epics.json", epics_json).map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Saved {} EPICs to Data/market_epics.json", epics.len());

    // Process EPICs one at a time to be extremely conservative with API usage
    for (index, epic) in epics.iter().enumerate() {
        // Process one EPIC at a time
        let epics_chunk = &[epic.clone()];

        info!(
            "Fetching market details for EPIC {}/{}: {}",
            index + 1,
            total_epics,
            epic
        );

        match market_service
            .get_multiple_market_details(&session, epics_chunk)
            .await
        {
            Ok(details_vec) => {
                // Match each result with its corresponding EPIC
                for (i, details) in details_vec.iter().enumerate() {
                    let epic = &epics_chunk[i];
                    debug!("✅ Successfully fetched details for {}", epic);
                    let (mid, spread) =
                        if details.snapshot.bid.is_none() || details.snapshot.offer.is_none() {
                            warn!("❌ Missing bid or offer for {}", epic);
                            (None, None)
                        } else {
                            let mid = details.snapshot.offer.unwrap()
                                + details.snapshot.bid.unwrap() / 2.0;
                            let spread =
                                details.snapshot.offer.unwrap() - details.snapshot.bid.unwrap();
                            (Some(mid), Some(spread))
                        };
                    let last_dealing_date = match details.instrument.expiry_details.clone() {
                        Some(expiry_details) => expiry_details.last_dealing_date,
                        None => details.instrument.expiry.clone(),
                    };

                    let market_summary = MarketSummary {
                        instrument_name: details.instrument.name.clone(),
                        epic: epic.clone(),
                        bid: details.snapshot.bid,
                        offer: details.snapshot.offer,
                        mid,
                        spread,
                        last_dealing_date,
                    };
                    market_summaries.push(market_summary);
                }

                processed_count += details_vec.len();
                info!(
                    "✅ Successfully processed batch of {} EPICs ({}/{})",
                    details_vec.len(),
                    processed_count,
                    total_epics
                );
            }
            Err(e) => {
                error!("❌ Failed to fetch details for batch: {:?}", e);

                // Fall back to processing EPICs individually in case of batch failure
                info!("Falling back to processing EPICs individually...");

                for epic in epics_chunk {
                    info!("Fetching market details for {} individually", epic);

                    match market_service.get_market_details(&session, epic).await {
                        Ok(details) => {
                            debug!("✅ Successfully fetched details for {}", epic);
                            let (mid, spread) = if details.snapshot.bid.is_none()
                                || details.snapshot.offer.is_none()
                            {
                                warn!("❌ Missing bid or offer for {}", epic);
                                (None, None)
                            } else {
                                let mid = details.snapshot.offer.unwrap()
                                    + details.snapshot.bid.unwrap() / 2.0;
                                let spread =
                                    details.snapshot.offer.unwrap() - details.snapshot.bid.unwrap();
                                (Some(mid), Some(spread))
                            };
                            let last_dealing_date = match details.instrument.expiry_details.clone()
                            {
                                Some(expiry_details) => expiry_details.last_dealing_date,
                                None => details.instrument.expiry.clone(),
                            };

                            let market_summary = MarketSummary {
                                instrument_name: details.instrument.name,
                                epic: epic.clone(),
                                bid: details.snapshot.bid,
                                offer: details.snapshot.offer,
                                mid,
                                spread,
                                last_dealing_date,
                            };
                            market_summaries.push(market_summary);
                            processed_count += 1;
                        }
                        Err(e) => {
                            error!("❌ Failed to fetch details for {}: {:?}", epic, e);
                        }
                    }
                }
            }
        }

        // Save intermediate results after each batch
        if processed_count > 0 {
            info!(
                "Processed {}/{} EPICs. Saving intermediate results...",
                processed_count,
                epics.len()
            );
        }
    }

    Ok(market_summaries)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    // Get market summaries
    let market_summaries = get_market_summaries().await?;

    // Save the results to a JSON file
    if !market_summaries.is_empty() {
        let json_data = market_summaries
            .iter()
            .map(|summary| {
                serde_json::json!({
                    "instrument_name": summary.instrument_name,
                    "epic": summary.epic,
                    "bid": summary.bid,
                    "offer": summary.offer,
                    "mid": summary.mid,
                    "spread": summary.spread,
                    "last_dealing_date": summary.last_dealing_date
                })
            })
            .collect::<Vec<_>>();

        let json =
            serde_json::to_string_pretty(&json_data).map_err(|e| Box::new(e) as Box<dyn Error>)?;

        fs::write("Data/market_summary.json", &json).map_err(|e| Box::new(e) as Box<dyn Error>)?;
        info!("Saved market summary data to Data/market_summary.json");
    }

    // Display the results
    info!(
        "\n{:<30} {:<20} {:<10} {:<10} {:<10} {:<10} {:<20}",
        "INSTRUMENT NAME", "EPIC", "BID", "OFFER", "MID", "SPREAD", "LAST DEALING DATE"
    );
    info!("{:-<110}", "");

    for summary in &market_summaries {
        info!(
            "{:<30} {:<20} {:<10.2} {:<10.2} {:<10.2} {:<10.2} {:<20}",
            summary.instrument_name,
            summary.epic,
            summary.bid.unwrap_or(0.0),
            summary.offer.unwrap_or(0.0),
            summary.mid.unwrap_or(0.0),
            summary.spread.unwrap_or(0.0),
            summary.last_dealing_date
        );
    }

    info!(
        "Processed {} market summaries successfully",
        market_summaries.len()
    );

    Ok(())
}
