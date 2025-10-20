use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::presentation::{build_market_hierarchy, extract_markets_from_hierarchy};
use ig_client::utils::logger::setup_logger;
use ig_client::utils::parsing::{normalize_text, parse_instrument_name};
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    config::Config, session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl,
};
use std::{error::Error, sync::Arc};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configure logger
    setup_logger();

    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    info!("Loaded configuration → {}", config.rest_api.base_url);

    // Create HTTP client
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));

    // Create market service
    let market_service = MarketServiceImpl::new(config.clone(), client);

    // Create authenticator
    let auth = IgAuth::new(&config);

    // Login
    info!("Logging in...");
    let session = match auth.login().await {
        Ok(session) => {
            info!("Login successful. Account ID: {}", session.account_id);
            session
        }
        Err(e) => {
            error!("Login failed: {:?}", e);
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };

    // Create a directory for the output file if it doesn't exist
    std::fs::create_dir_all("Data").map_err(|e| Box::new(e) as Box<dyn Error>)?;

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

    // Build the complete market hierarchy
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
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };

    // Extract all markets from the hierarchy into a flat list
    let markets = extract_markets_from_hierarchy(&hierarchy);
    info!("Extracted {} markets from the hierarchy", markets.len());

    // Save the complete data to a JSON file
    let json_data = markets
        .iter()
        .map(|market| {
            let parsed_info = parse_instrument_name(&market.instrument_name);
            let normalized_asset_name = normalize_text(&parsed_info.asset_name);

            // Create a JSON object with all fields
            serde_json::json!({
                "epic": market.epic,
                "instrument_name": market.instrument_name,
                "expiry": market.expiry,
                "asset_name": normalized_asset_name,
                "strike": parsed_info.strike,
                "option_type": parsed_info.option_type
            })
        })
        .collect::<Vec<_>>();

    let json = match serde_json::to_string_pretty(&json_data) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize to JSON: {:?}", e);
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };

    let filename = "Data/market_table.json";
    if let Err(e) = std::fs::write(filename, &json) {
        error!("Failed to write to file {}: {:?}", filename, e);
        return Err(Box::new(e) as Box<dyn Error>);
    }
    info!("Complete market data saved to '{}'", filename);
    Ok(())
}
