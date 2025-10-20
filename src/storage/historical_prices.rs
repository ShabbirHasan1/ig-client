use crate::presentation::market::HistoricalPrice;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::{info, warn};

/// Initialize the historical_prices table in PostgreSQL
pub async fn initialize_historical_prices_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Initializing historical_prices table...");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS historical_prices (
            id BIGSERIAL PRIMARY KEY,
            epic VARCHAR(255) NOT NULL,
            snapshot_time TIMESTAMPTZ NOT NULL,
            open_bid DOUBLE PRECISION,
            open_ask DOUBLE PRECISION,
            open_last_traded DOUBLE PRECISION,
            high_bid DOUBLE PRECISION,
            high_ask DOUBLE PRECISION,
            high_last_traded DOUBLE PRECISION,
            low_bid DOUBLE PRECISION,
            low_ask DOUBLE PRECISION,
            low_last_traded DOUBLE PRECISION,
            close_bid DOUBLE PRECISION,
            close_ask DOUBLE PRECISION,
            close_last_traded DOUBLE PRECISION,
            last_traded_volume BIGINT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(epic, snapshot_time)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create index for better query performance
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_historical_prices_epic_time 
        ON historical_prices(epic, snapshot_time DESC)
        "#,
    )
    .execute(pool)
    .await?;

    // Create trigger for updating updated_at timestamp
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION update_updated_at_column()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = NOW();
            RETURN NEW;
        END;
        $$ language 'plpgsql'
        "#,
    )
    .execute(pool)
    .await?;

    // Drop existing trigger if it exists
    sqlx::query(
        r#"
        DROP TRIGGER IF EXISTS update_historical_prices_updated_at ON historical_prices
        "#,
    )
    .execute(pool)
    .await?;

    // Create the trigger
    sqlx::query(
        r#"
        CREATE TRIGGER update_historical_prices_updated_at
            BEFORE UPDATE ON historical_prices
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column()
        "#,
    )
    .execute(pool)
    .await?;

    info!("✅ Historical prices table initialized successfully");
    Ok(())
}

/// Storage statistics for tracking insert/update operations
#[derive(Debug, Default)]
pub struct StorageStats {
    /// Number of new records inserted into the database
    pub inserted: usize,
    /// Number of existing records updated in the database
    pub updated: usize,
    /// Number of records skipped due to errors or validation issues
    pub skipped: usize,
    /// Total number of records processed (inserted + updated + skipped)
    pub total_processed: usize,
}

/// Store historical prices in PostgreSQL with UPSERT logic
pub async fn store_historical_prices(
    pool: &PgPool,
    epic: &str,
    prices: &[HistoricalPrice],
) -> Result<StorageStats, sqlx::Error> {
    let mut stats = StorageStats::default();
    let mut tx = pool.begin().await?;

    info!(
        "Processing {} price records for epic: {}",
        prices.len(),
        epic
    );

    for (i, price) in prices.iter().enumerate() {
        stats.total_processed += 1;

        // Parse snapshot time
        let snapshot_time = match parse_snapshot_time(&price.snapshot_time) {
            Ok(time) => time,
            Err(e) => {
                warn!(
                    "⚠️  Skipping record {}: Invalid timestamp '{}': {}",
                    i + 1,
                    price.snapshot_time,
                    e
                );
                stats.skipped += 1;
                continue;
            }
        };

        // Use UPSERT (INSERT ... ON CONFLICT ... DO UPDATE)
        let result = sqlx::query(
            r#"
            INSERT INTO historical_prices (
                epic, snapshot_time,
                open_bid, open_ask, open_last_traded,
                high_bid, high_ask, high_last_traded,
                low_bid, low_ask, low_last_traded,
                close_bid, close_ask, close_last_traded,
                last_traded_volume
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (epic, snapshot_time) 
            DO UPDATE SET
                open_bid = EXCLUDED.open_bid,
                open_ask = EXCLUDED.open_ask,
                open_last_traded = EXCLUDED.open_last_traded,
                high_bid = EXCLUDED.high_bid,
                high_ask = EXCLUDED.high_ask,
                high_last_traded = EXCLUDED.high_last_traded,
                low_bid = EXCLUDED.low_bid,
                low_ask = EXCLUDED.low_ask,
                low_last_traded = EXCLUDED.low_last_traded,
                close_bid = EXCLUDED.close_bid,
                close_ask = EXCLUDED.close_ask,
                close_last_traded = EXCLUDED.close_last_traded,
                last_traded_volume = EXCLUDED.last_traded_volume,
                updated_at = NOW()
            "#,
        )
        .bind(epic)
        .bind(snapshot_time)
        .bind(price.open_price.bid)
        .bind(price.open_price.ask)
        .bind(price.open_price.last_traded)
        .bind(price.high_price.bid)
        .bind(price.high_price.ask)
        .bind(price.high_price.last_traded)
        .bind(price.low_price.bid)
        .bind(price.low_price.ask)
        .bind(price.low_price.last_traded)
        .bind(price.close_price.bid)
        .bind(price.close_price.ask)
        .bind(price.close_price.last_traded)
        .bind(price.last_traded_volume)
        .execute(&mut *tx)
        .await?;

        // Check if it was an insert or update
        if result.rows_affected() > 0 {
            // Query to check if this was an insert or update
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM historical_prices WHERE epic = $1 AND snapshot_time = $2 AND created_at = updated_at"
            )
                .bind(epic)
                .bind(snapshot_time)
                .fetch_one(&mut *tx)
                .await?;

            if count > 0 {
                stats.inserted += 1;
            } else {
                stats.updated += 1;
            }
        } else {
            stats.skipped += 1;
        }

        // Log progress every 100 records
        if (i + 1) % 100 == 0 {
            info!("  Processed {}/{} records...", i + 1, prices.len());
        }
    }

    tx.commit().await?;
    info!("✅ Transaction committed successfully");

    Ok(stats)
}

/// Parse snapshot time from IG format to `DateTime<Utc>`
fn parse_snapshot_time(snapshot_time: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    // IG format: "yyyy/MM/dd hh:mm:ss" or "yyyy-MM-dd hh:mm:ss"
    let formats = [
        "%Y/%m/%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y-%m-%d %H:%M",
    ];

    for format in &formats {
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(snapshot_time, format) {
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }
    }

    Err(format!("Unable to parse timestamp: {}", snapshot_time).into())
}

/// Database statistics for a specific epic
#[derive(Debug)]
pub struct TableStats {
    /// Total number of records in the database for this epic
    pub total_records: i64,
    /// Earliest date in the dataset (formatted as string)
    pub earliest_date: String,
    /// Latest date in the dataset (formatted as string)
    pub latest_date: String,
    /// Average closing price across all records
    pub avg_close_price: f64,
    /// Minimum price (lowest of all low prices) in the dataset
    pub min_price: f64,
    /// Maximum price (highest of all high prices) in the dataset
    pub max_price: f64,
}

/// Get statistics for the historical_prices table
pub async fn get_table_statistics(pool: &PgPool, epic: &str) -> Result<TableStats, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT 
            COUNT(*) as total_records,
            MIN(snapshot_time)::text as earliest_date,
            MAX(snapshot_time)::text as latest_date,
            AVG(close_bid) as avg_close_price,
            MIN(LEAST(low_bid, low_ask)) as min_price,
            MAX(GREATEST(high_bid, high_ask)) as max_price
        FROM historical_prices 
        WHERE epic = $1
        "#,
    )
    .bind(epic)
    .fetch_one(pool)
    .await?;

    Ok(TableStats {
        total_records: row.get("total_records"),
        earliest_date: row.get("earliest_date"),
        latest_date: row.get("latest_date"),
        avg_close_price: row.get::<Option<f64>, _>("avg_close_price").unwrap_or(0.0),
        min_price: row.get::<Option<f64>, _>("min_price").unwrap_or(0.0),
        max_price: row.get::<Option<f64>, _>("max_price").unwrap_or(0.0),
    })
}
