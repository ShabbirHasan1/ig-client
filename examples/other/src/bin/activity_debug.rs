use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    config::Config, session::auth::IgAuth, session::interface::IgAuthenticator,
    utils::logger::setup_logger,
};
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Create configuration using the default Config implementation
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    info!("Configuration loaded");

    // Create authenticator
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    // Login to IG
    info!("Logging in to IG...");
    let session = authenticator.login().await?;
    info!("Session started successfully");

    // Get activity with raw response handling
    info!("Fetching account activity...");
    let url = format!(
        "{}/{}",
        config.rest_api.base_url.trim_end_matches('/'),
        "history/activity?from=2025-03-01T00:00:00Z&to=2025-04-01T00:00:00Z&detailed=true"
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("X-IG-API-KEY", &config.credentials.api_key)
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("Accept", "application/json; charset=UTF-8")
        .header("Version", "3")
        .header("CST", &session.cst)
        .header("X-SECURITY-TOKEN", &session.token)
        .send()
        .await?;

    if response.status().is_success() {
        // Get the raw text response to see the actual structure
        let text = response.text().await?;
        info!("Raw API response: {}", text);
    } else {
        error!("Request failed with status: {}", response.status());
        let error_text = response.text().await?;
        error!("Error response: {}", error_text);
    }

    Ok(())
}
