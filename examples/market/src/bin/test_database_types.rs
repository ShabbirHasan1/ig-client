use ig_client::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Initialize simple logging
    info!("Testing database types compatibility...");

    // Create database connection
    let db_config = create_database_config_from_env()?;
    let pool = create_connection_pool(&db_config).await?;

    // Create database service
    let db_service = MarketDatabaseService::new(pool, "IG".to_string());

    // Initialize database
    db_service.initialize_database().await?;
    info!("Database initialized successfully");

    // Test searching (this will test f64 <-> DOUBLE PRECISION compatibility)
    info!("Testing search functionality...");
    let search_results = db_service.search_instruments("Germany").await?;
    info!(
        "Search completed successfully. Found {} results",
        search_results.len()
    );

    for result in &search_results {
        info!(
            "Found instrument: {} - {} (bid: {:?}, offer: {:?})",
            result.epic, result.instrument_name, result.bid, result.offer
        );
    }

    // Test statistics
    let stats = db_service.get_statistics().await?;
    info!(
        "Database statistics: {} nodes, {} instruments",
        stats.node_count, stats.instrument_count
    );

    info!("All tests completed successfully! f64 <-> DOUBLE PRECISION compatibility confirmed.");

    Ok(())
}
