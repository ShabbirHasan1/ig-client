use ig_client::storage::market_database::MarketDatabaseService;
use ig_client::storage::utils::{create_connection_pool, create_database_config_from_env};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize simple logging
    println!("Testing database types compatibility...");

    // Create database connection
    let db_config = create_database_config_from_env()?;
    let pool = create_connection_pool(&db_config).await?;

    // Create database service
    let db_service = MarketDatabaseService::new(pool, "IG".to_string());

    // Initialize database
    db_service.initialize_database().await?;
    println!("Database initialized successfully");

    // Test searching (this will test f64 <-> DOUBLE PRECISION compatibility)
    println!("Testing search functionality...");
    let search_results = db_service.search_instruments("Germany").await?;
    println!(
        "Search completed successfully. Found {} results",
        search_results.len()
    );

    for result in &search_results {
        println!(
            "Found instrument: {} - {} (bid: {:?}, offer: {:?})",
            result.epic, result.instrument_name, result.bid, result.offer
        );
    }

    // Test statistics
    let stats = db_service.get_statistics().await?;
    println!(
        "Database statistics: {} nodes, {} instruments",
        stats.node_count, stats.instrument_count
    );

    println!("All tests completed successfully! f64 <-> DOUBLE PRECISION compatibility confirmed.");

    Ok(())
}
