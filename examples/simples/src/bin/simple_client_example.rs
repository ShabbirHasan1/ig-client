use ig_client::application::client::Client;
use ig_client::application::interfaces::market::MarketService;
use ig_client::model::http::HttpClient;
use ig_client::utils::setup_logger;
/// Example demonstrating the new simplified Client API
///
/// This example shows how easy it is to use the new Client that handles
/// all authentication and token refresh automatically.
///
/// Run with: cargo run --example simple_client_example
use serde_json::Value;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    setup_logger();

    // Load environment variables
    info!("Starting simple client example");

    // Create client - authentication happens automatically
    info!("Creating client and authenticating...");
    let client = Client::default();
    info!("✓ Client created and authenticated");

    let epic = "OP.D.OTCGC3.4050C.IP";
    let market = client.get_market_details(epic).await?;

    info!("Market details: {:#?}", market);

    Ok(())
}
