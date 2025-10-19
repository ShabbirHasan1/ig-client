use ig_client::application::client::Client;
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

    // Get session info
    let session = client.get_session().await?;
    info!("✓ Logged in as account: {}", session.account_id);
    if session.is_oauth() {
        info!("  Using OAuth (API v3)");
    } else {
        info!("  Using CST/X-SECURITY-TOKEN (API v2)");
    }

    // Make API calls - token refresh is automatic
    info!("\n=== Making API calls ===\n");

    // Example 1: Get specific market details
    info!("1. Getting market details for OP.D.OTCGC3.4050C.IP...");
    let epic = "OP.D.OTCGC3.4050C.IP";
    let market_path = format!("/markets/{}", epic);
    let market: Value = client.get(&market_path).await?;

    info!("Market details: {:#?}", market);

    Ok(())
}
