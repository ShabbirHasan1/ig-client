use ig_client::application::models::market::MarketNode;
use ig_client::storage::market_database::MarketDatabaseService;
use ig_client::storage::utils::{create_connection_pool, create_database_config_from_env};
use ig_client::utils::logger::setup_logger;
use std::collections::HashMap;
use std::{error::Error, fs};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    setup_logger();

    info!("Starting filtered market storage example...");

    // Load and deserialize the market hierarchy backup JSON file
    let json_path = "Data/market_hierarchy_backup.json";
    info!("Loading market hierarchy from: {}", json_path);

    let json_content =
        fs::read_to_string(json_path).map_err(|e| format!("Failed to read JSON file: {}", e))?;

    info!(
        "Successfully loaded JSON file ({} bytes)",
        json_content.len()
    );

    // Deserialize JSON to Vec<MarketNode>
    let hierarchy: Vec<MarketNode> = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to deserialize JSON: {}", e))?;

    info!(
        "Successfully deserialized {} top-level market nodes",
        hierarchy.len()
    );

    // Create the symbol mapping HashMap as specified
    let symbol_map: HashMap<&str, &str> = HashMap::from([
        ("germany 40", "GER40"),
        ("us 500", "US500"),
        ("wall street", "US30"),
        ("us tech 100", "USTECH"),
        ("france 40", "FRA40"),
        ("eu stocks 50", "EU50"),
        ("japan 225", "NI225"),
        ("uk 100", "UK100"),
        ("ftse", "UK100"),
        ("us crude", "OIL"),
        ("oil", "OIL"),
        ("natural gas", "NATGAS"),
        ("spot gold", "GOLD"),
        ("gold", "GOLD"),
        ("spot silver", "SILVER"),
        ("silver", "SILVER"),
        ("bitcoin", "BITCOIN"),
        ("ether", "ETHEREUM"),
        ("volatility index", "VIX"),
        ("pago mediante tarjeta", "DEPOSIT"),
        ("dinero depositado en su cuenta", "DEPOSIT"),
        ("funds transfer", "DEPOSIT"),
        ("Australia 200", "AUS200"),
        ("Japón 225", "NI225"),
        ("AUDUSD", "AUDUSD"),
        ("EURUSD", "EURUSD"),
        ("GBPUSD", "GBPUSD"),
        ("USDCAD", "USDCAD"),
        ("EURGBP", "EURGBP"),
        ("GBPJPY", "GBPJPY"),
        ("USDJPY", "USDJPY"),
        ("EURJPY", "EURJPY"),
        ("USDCHF", "USDCHF"),
    ]);

    info!("Created symbol mapping with {} entries", symbol_map.len());

    // Set up database connection
    let db_config = create_database_config_from_env()
        .map_err(|e| format!("Failed to create database config: {}", e))?;

    info!("Database config created successfully");

    let pool = create_connection_pool(&db_config)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    info!("Successfully connected to PostgreSQL database");

    // Create database service
    let db_service = MarketDatabaseService::new(pool, "IG".to_string());

    info!("Database service created successfully");

    // Define the custom table name
    let table_name = "filtered_market_instruments";

    // Store filtered market nodes using the new method
    info!("Storing filtered market nodes to table '{}'...", table_name);

    db_service
        .store_filtered_market_nodes(&hierarchy, &symbol_map, table_name)
        .await
        .map_err(|e| format!("Failed to store filtered market nodes: {}", e))?;

    info!("✅ Successfully stored filtered market instruments!");
    info!("Filtered market storage example completed successfully!");

    Ok(())
}
