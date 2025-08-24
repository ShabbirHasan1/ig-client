use crate::application::models::transaction::StoreTransaction;
use crate::error::AppError;
use crate::storage::config::DatabaseConfig;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json;
use sqlx::{Executor, PgPool};
use tracing::info;

/// Stores a list of transactions in the database
///
/// # Arguments
/// * `pool` - PostgreSQL connection pool
/// * `txs` - List of transactions to store
///
/// # Returns
/// * `Result<usize, AppError>` - Number of transactions inserted or an error
pub async fn store_transactions(
    pool: &sqlx::PgPool,
    txs: &[StoreTransaction],
) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    let mut inserted = 0;

    for t in txs {
        let result = tx
            .execute(
                sqlx::query(
                    r#"
                    INSERT INTO ig_options (
                        reference, deal_date, underlying, strike,
                        option_type, expiry, transaction_type, pnl_eur, is_fee, raw
                    )
                    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                    ON CONFLICT (raw_hash) DO NOTHING
                    "#,
                )
                .bind(&t.reference)
                .bind(t.deal_date)
                .bind(&t.underlying)
                .bind(t.strike)
                .bind(&t.option_type)
                .bind(t.expiry)
                .bind(&t.transaction_type)
                .bind(t.pnl_eur)
                .bind(t.is_fee)
                .bind(&t.raw_json),
            )
            .await?;

        inserted += result.rows_affected() as usize;
    }

    tx.commit().await?;
    Ok(inserted)
}

/// Serializes a value to a JSON string
pub fn serialize_to_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserializes a JSON string into a value
pub fn deserialize_from_json<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(s)
}

/// Creates a PostgreSQL connection pool from database configuration
///
/// # Arguments
/// * `config` - Database configuration containing URL and max connections
///
/// # Returns
/// * `Result<PgPool, AppError>` - Connection pool or an error
pub async fn create_connection_pool(config: &DatabaseConfig) -> Result<PgPool, AppError> {
    info!(
        "Creating PostgreSQL connection pool with max {} connections",
        config.max_connections
    );

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(AppError::Db)?;

    info!("PostgreSQL connection pool created successfully");
    Ok(pool)
}

/// Creates a database configuration from environment variables
///
/// # Returns
/// * `Result<DatabaseConfig, AppError>` - Database configuration or an error
pub fn create_database_config_from_env() -> Result<DatabaseConfig, AppError> {
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL").map_err(|_| {
        AppError::InvalidInput("DATABASE_URL environment variable is required".to_string())
    })?;

    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u32>()
        .map_err(|e| {
            AppError::InvalidInput(format!("Invalid DATABASE_MAX_CONNECTIONS value: {e}"))
        })?;

    Ok(DatabaseConfig {
        url,
        max_connections,
    })
}
