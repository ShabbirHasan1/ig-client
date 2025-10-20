use ig_client::prelude::*;
use ig_client::presentation::transaction::TransactionList;
use ig_client::storage::utils::store_transactions;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    info!("=== IG Transactions Example ===");

    // Create client
    let client = Client::default();

    // Get config for database connection
    let config = Config::default();

    // Get open transactions
    info!("Fetching open transactions...");
    let transactions = match client
        .get_transactions("2024-07-01T00:00:00Z", "2025-07-13T23:59:59Z")
        .await
    {
        Ok(transactions) => transactions,
        Err(e) => {
            error!("Failed to get transactions: {}", e);
            return Err(Box::<dyn std::error::Error>::from(format!(
                "Failed to get transactions: {}",
                e
            )));
        }
    };

    if transactions.transactions.is_empty() {
        info!("No open transactions currently");
    } else {
        info!("Open transactions: {}", transactions.transactions.len());

        for (i, transaction) in transactions.transactions.iter().enumerate() {
            // Log the transaction as pretty JSON
            // info!(
            //     "Transactions #{}: {}",
            //     i + 1,
            //     serde_json::to_string_pretty(&serde_json::to_value(transaction).unwrap()).unwrap()
            // );
            info!("Transactions #{}: {}", i + 1, transaction.instrument_name);
        }
    }
    // Store the transactions in database if configured
    if let Ok(pool) = config.pg_pool().await {
        let tx_list = TransactionList::from(&transactions.transactions);
        let inserted = store_transactions(&pool, tx_list.as_ref()).await?;
        info!("Inserted {} rows into database", inserted);
    } else {
        info!("Database not configured, skipping storage");
    }

    Ok(())
}
