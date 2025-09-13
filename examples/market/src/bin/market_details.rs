use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    application::services::MarketService, config::Config, session::auth::IgAuth,
    session::interface::IgAuthenticator, transport::http_client::IgHttpClientImpl,
    utils::logger::setup_logger,
};
use std::{error::Error, sync::Arc};
use tracing::{debug, error, info};

// Constants for API request handling
const BATCH_SIZE: usize = 25; // Number of EPICs to process before saving results

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    setup_logger();

    // Load configuration
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    info!("Loaded configuration → {}", config.rest_api.base_url);

    // Create the HTTP client
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));

    // Create the authenticator
    let auth = IgAuth::new(&config);

    // Create the market service
    let market_service = MarketServiceImpl::new(config.clone(), client);

    // Login to get a session
    info!("Logging in...");
    let session = auth
        .login()
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Login successful. Account ID: {}", session.account_id);

    // Check if we need to switch accounts
    let session = if !config.credentials.account_id.is_empty()
        && session.account_id != config.credentials.account_id
    {
        info!("Switching to account: {}", config.credentials.account_id);
        match auth
            .switch_account(&session, &config.credentials.account_id, Some(true))
            .await
        {
            Ok(new_session) => {
                info!("✅ Switched to account: {}", new_session.account_id);
                new_session
            }
            Err(e) => {
                error!(
                    "Could not switch to account {}: {:?}. Continuing with current account.",
                    config.credentials.account_id, e
                );
                session
            }
        }
    } else {
        session
    };

    // Get the EPICs from command line arguments or use the default range
    let epics_arg = std::env::args().nth(1);

    let epics = match epics_arg {
        Some(arg) => {
            // Parse comma-separated list of EPICs
            arg.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        }
        None => {
            // Generate the default range of EPICs
            info!(
                "No EPICs provided, using default range from DO.D.OTCDDAX.1.IP to DO.D.OTCDDAX.196.IP"
            );
            (1..=196)
                .map(|i| format!("DO.D.OTCDDAX.{}.IP", i))
                .collect::<Vec<String>>()
        }
    };

    info!("Fetching market details for {} EPICs", epics.len());

    // Create a vector to store all the market details
    let mut market_details_vec = Vec::new();

    // Process EPICs in batches to use the API more efficiently
    let mut processed_count = 0;
    let total_epics = epics.len();

    // Process EPICs in batches of BATCH_SIZE
    for chunk_start in (0..epics.len()).step_by(BATCH_SIZE) {
        let chunk_end = std::cmp::min(chunk_start + BATCH_SIZE, epics.len());
        let epics_chunk = &epics[chunk_start..chunk_end];

        info!(
            "Fetching market details for batch {}/{} (EPICs {}-{} of {})",
            (chunk_start / BATCH_SIZE) + 1,
            (total_epics + BATCH_SIZE - 1) / BATCH_SIZE,
            chunk_start + 1,
            chunk_end,
            total_epics
        );

        // Log the EPICs being processed in this batch
        debug!("EPICs in this batch: {}", epics_chunk.join(", "));

        match market_service
            .get_multiple_market_details(&session, epics_chunk)
            .await
        {
            Ok(details_vec) => {
                // Match each result with its corresponding EPIC
                for (i, details) in details_vec.iter().enumerate() {
                    let epic = &epics_chunk[i];
                    debug!("✅ Successfully fetched details for {}", epic);
                    market_details_vec.push((epic.clone(), details.clone()));
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
                            market_details_vec.push((epic.clone(), details));
                            processed_count += 1;
                        }
                        Err(e) => {
                            error!("❌ Failed to fetch details for {}: {:?}", epic, e);
                        }
                    }
                }
            }
        }
    }

    // Display the results in a table format
    info!(
        "\n{:<40} {:<15} {:<10} {:<10} {:<10} {:<10} {:<20} {:<15}",
        "INSTRUMENT NAME", "EPIC", "BID", "OFFER", "MID", "SPREAD", "LAST DEALING DATE", "HIGH/LOW"
    );
    info!(
        "{:-<40} {:-<15} {:-<10} {:-<10} {:-<10} {:-<10} {:-<20} {:-<15}",
        "", "", "", "", "", "", "", ""
    );

    // Sort the results by instrument name for better readability
    market_details_vec.sort_by(|(_, a), (_, b)| {
        a.instrument
            .name
            .to_lowercase()
            .cmp(&b.instrument.name.to_lowercase())
    });

    // Save the final results to JSON
    let json_data = market_details_vec.iter()
        .map(|(epic, details)| {
            serde_json::json!({
                "epic": epic,
                "instrument_name": details.instrument.name,
                "expiry": details.instrument.expiry,
                "last_dealing_date": details.instrument.expiry_details.as_ref().map(|ed| ed.last_dealing_date.clone()),
                "bid": details.snapshot.bid,
                "offer": details.snapshot.offer,
                "high": details.snapshot.high,
                "low": details.snapshot.low,
                "update_time": details.snapshot.update_time
            })
        })
        .collect::<Vec<_>>();

    let json =
        serde_json::to_string_pretty(&json_data).map_err(|e| Box::new(e) as Box<dyn Error>)?;

    // Display the results in the console
    for (epic, details) in &market_details_vec {
        // Calculate MID and SPREAD values
        let mid = match (details.snapshot.bid, details.snapshot.offer) {
            (Some(bid), Some(offer)) => Some((bid + offer) / 2.0),
            _ => None,
        };

        let spread = match (details.snapshot.bid, details.snapshot.offer) {
            (Some(bid), Some(offer)) => Some(offer - bid),
            _ => None,
        };

        // Format high/low as "high/low"
        let high_low = format!(
            "{}/{}",
            details
                .snapshot
                .high
                .map(|h| h.to_string())
                .unwrap_or_else(|| "-".to_string()),
            details
                .snapshot
                .low
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string())
        );

        // Get the last dealing date from expiry_details if available
        let last_dealing_date = details
            .instrument
            .expiry_details
            .as_ref()
            .map(|ed| truncate(&ed.last_dealing_date, 18))
            .unwrap_or_else(|| truncate(&details.instrument.expiry, 18));

        info!(
            "{:<40} {:<15} {:<10} {:<10} {:<10} {:<10} {:<20} {:<15}",
            truncate(&details.instrument.name, 38),
            truncate(epic, 13),
            details
                .snapshot
                .bid
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".to_string()),
            details
                .snapshot
                .offer
                .map(|o| o.to_string())
                .unwrap_or_else(|| "-".to_string()),
            mid.map(|m| format!("{:.2}", m))
                .unwrap_or_else(|| "-".to_string()),
            spread
                .map(|s| format!("{:.2}", s))
                .unwrap_or_else(|| "-".to_string()),
            last_dealing_date,
            high_low
        );
    }

    // Save the results to a file
    let filename = format!("Data/market_details.json");
    std::fs::write(&filename, &json).map_err(|e| Box::new(e) as Box<dyn Error>)?;
    info!("Results saved to '{}'", filename);
    info!(
        "Successfully processed {} out of {} EPICs",
        market_details_vec.len(),
        epics.len()
    );

    Ok(())
}

// Helper function to truncate strings to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len - 3])
    }
}
