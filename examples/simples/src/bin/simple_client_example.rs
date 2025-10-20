use ig_client::application::client::Client;
use ig_client::application::interfaces::market::MarketService;
use ig_client::utils::setup_logger;
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
