use ig_client::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    info!("=== IG Working Orders Example ===");

    // Create client
    let client = Client::default();

    // Get working orders
    info!("Fetching working orders...");
    let working_orders = client.get_working_orders().await?;

    if working_orders.working_orders.is_empty() {
        info!("No working orders currently");
    } else {
        info!("Working orders: {}", working_orders.working_orders.len());

        // Display details of each working order as JSON
        for (i, order) in working_orders.working_orders.iter().enumerate() {
            // Log the working order as pretty JSON
            info!(
                "Working Order #{}: {}",
                i + 1,
                serde_json::to_string_pretty(&serde_json::to_value(order).unwrap()).unwrap()
            );
        }
    }

    Ok(())
}
