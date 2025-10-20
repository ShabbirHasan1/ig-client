use ig_client::prelude::*;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    info!("=== Activity Example ===");

    // Create client
    let client = Client::default();

    // Get account activity with detailed information
    info!("Fetching account activity with details...");
    let activity = match client
        .get_activity_with_details("2025-03-01T00:00:00Z", "2025-04-01T00:00:00Z")
        .await
    {
        Ok(activity) => activity,
        Err(e) => {
            error!("Failed to get activity: {}", e);
            return Err(Box::<dyn std::error::Error>::from(format!(
                "Failed to get activity: {e}"
            )));
        }
    };

    if activity.activities.is_empty() {
        info!("No activities found for the specified period");
    } else {
        info!("Activities found: {}", activity.activities.len());

        // Display activities with detailed information
        for (i, activity_item) in activity.activities.iter().enumerate() {
            // Log the activity as pretty JSON
            info!(
                "Activity #{}: {}",
                i + 1,
                serde_json::to_string_pretty(&serde_json::to_value(activity_item).unwrap())
                    .unwrap()
            );

            info!("---"); // Separator between activities
        }
    }

    Ok(())
}
