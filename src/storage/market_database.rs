use crate::application::models::market::{MarketData, MarketNode};
use crate::storage::market_persistence::{MarketHierarchyNode, MarketInstrument};
use sqlx::{Executor, PgPool, Row};
use tracing::info;

/// Service for managing market data persistence in PostgreSQL
pub struct MarketDatabaseService {
    pool: PgPool,
    exchange_name: String,
}

impl MarketDatabaseService {
    /// Creates a new MarketDatabaseService
    pub fn new(pool: PgPool, exchange_name: String) -> Self {
        Self {
            pool,
            exchange_name,
        }
    }

    /// Initializes the database tables and triggers
    pub async fn initialize_database(&self) -> Result<(), sqlx::Error> {
        info!("Initializing market database tables...");

        // Create market_hierarchy_nodes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_hierarchy_nodes (
                id VARCHAR(255) PRIMARY KEY,
                name VARCHAR(500) NOT NULL,
                parent_id VARCHAR(255) REFERENCES market_hierarchy_nodes(id),
                exchange VARCHAR(50) NOT NULL,
                level INTEGER NOT NULL DEFAULT 0,
                path TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create market_instruments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_instruments (
                epic VARCHAR(255) PRIMARY KEY,
                instrument_name VARCHAR(500) NOT NULL,
                instrument_type VARCHAR(100) NOT NULL,
                node_id VARCHAR(255) NOT NULL REFERENCES market_hierarchy_nodes(id),
                exchange VARCHAR(50) NOT NULL,
                expiry VARCHAR(50) NOT NULL DEFAULT '',
                high_limit_price DECIMAL(20,8),
                low_limit_price DECIMAL(20,8),
                market_status VARCHAR(50) NOT NULL,
                net_change DECIMAL(20,8),
                percentage_change DECIMAL(10,4),
                update_time VARCHAR(50),
                update_time_utc TIMESTAMPTZ,
                bid DECIMAL(20,8),
                offer DECIMAL(20,8),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for market_hierarchy_nodes
        let hierarchy_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_market_hierarchy_parent_id ON market_hierarchy_nodes(parent_id)",
            "CREATE INDEX IF NOT EXISTS idx_market_hierarchy_exchange ON market_hierarchy_nodes(exchange)",
            "CREATE INDEX IF NOT EXISTS idx_market_hierarchy_level ON market_hierarchy_nodes(level)",
            "CREATE INDEX IF NOT EXISTS idx_market_hierarchy_path ON market_hierarchy_nodes USING gin(to_tsvector('english', path))",
            "CREATE INDEX IF NOT EXISTS idx_market_hierarchy_name ON market_hierarchy_nodes USING gin(to_tsvector('english', name))",
        ];

        for index_sql in hierarchy_indexes {
            sqlx::query(index_sql).execute(&self.pool).await?;
        }

        // Create indexes for market_instruments
        let instrument_indexes = [
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_node_id ON market_instruments(node_id)",
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_exchange ON market_instruments(exchange)",
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_type ON market_instruments(instrument_type)",
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_status ON market_instruments(market_status)",
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_name ON market_instruments USING gin(to_tsvector('english', instrument_name))",
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_epic ON market_instruments(epic)",
            "CREATE INDEX IF NOT EXISTS idx_market_instruments_expiry ON market_instruments(expiry)",
        ];

        for index_sql in instrument_indexes {
            sqlx::query(index_sql).execute(&self.pool).await?;
        }

        // Create update timestamp function
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
        .execute(&self.pool)
        .await?;

        // Create triggers
        sqlx::query("DROP TRIGGER IF EXISTS update_market_hierarchy_nodes_updated_at ON market_hierarchy_nodes")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER update_market_hierarchy_nodes_updated_at
                BEFORE UPDATE ON market_hierarchy_nodes
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column()
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("DROP TRIGGER IF EXISTS update_market_instruments_updated_at ON market_instruments")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TRIGGER update_market_instruments_updated_at
                BEFORE UPDATE ON market_instruments
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column()
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Market database tables initialized successfully");
        Ok(())
    }

    /// Stores the complete market hierarchy in the database
    pub async fn store_market_hierarchy(
        &self,
        hierarchy: &[MarketNode],
    ) -> Result<(), sqlx::Error> {
        info!(
            "Storing market hierarchy with {} top-level nodes",
            hierarchy.len()
        );

        // Start a transaction
        let mut tx = self.pool.begin().await?;

        // Clear existing data for this exchange
        sqlx::query("DELETE FROM market_instruments WHERE exchange = $1")
            .bind(&self.exchange_name)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM market_hierarchy_nodes WHERE exchange = $1")
            .bind(&self.exchange_name)
            .execute(&mut *tx)
            .await?;

        // Store hierarchy nodes and instruments
        let mut node_count = 0;
        let mut instrument_count = 0;

        for node in hierarchy {
            let (nodes, instruments) = self.process_node_recursive(node, None, 0, "").await?;
            node_count += nodes.len();
            instrument_count += instruments.len();

            // Insert nodes
            for node in nodes {
                self.insert_hierarchy_node(&mut tx, &node).await?;
            }

            // Insert instruments
            for instrument in instruments {
                self.insert_market_instrument(&mut tx, &instrument).await?;
            }
        }

        // Commit transaction
        tx.commit().await?;

        info!(
            "Successfully stored {} hierarchy nodes and {} instruments",
            node_count, instrument_count
        );
        Ok(())
    }

    /// Processes a node recursively to extract all nodes and instruments
    fn process_node_recursive<'a>(
        &'a self,
        node: &'a MarketNode,
        parent_id: Option<&'a str>,
        level: i32,
        parent_path: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<(Vec<MarketHierarchyNode>, Vec<MarketInstrument>), sqlx::Error>,
                > + 'a,
        >,
    > {
        Box::pin(async move {
            let mut all_nodes = Vec::new();
            let mut all_instruments = Vec::new();

            // Build path for current node
            let current_path = MarketHierarchyNode::build_path(
                if parent_path.is_empty() {
                    None
                } else {
                    Some(parent_path)
                },
                &node.name,
            );

            // Create current node
            let current_node = MarketHierarchyNode::new(
                node.id.clone(),
                node.name.clone(),
                parent_id.map(|s| s.to_string()),
                self.exchange_name.clone(),
                level,
                current_path.clone(),
            );

            all_nodes.push(current_node);

            // Process markets in this node
            for market in &node.markets {
                let mut instrument = self.convert_market_data_to_instrument(market, &node.id);
                instrument.parse_update_time_utc();
                all_instruments.push(instrument);
            }

            // Process child nodes recursively
            for child in &node.children {
                let (child_nodes, child_instruments) = self
                    .process_node_recursive(child, Some(&node.id), level + 1, &current_path)
                    .await?;
                all_nodes.extend(child_nodes);
                all_instruments.extend(child_instruments);
            }

            Ok((all_nodes, all_instruments))
        })
    }

    /// Converts MarketData to MarketInstrument
    fn convert_market_data_to_instrument(
        &self,
        market: &MarketData,
        node_id: &str,
    ) -> MarketInstrument {
        let mut instrument = MarketInstrument::new(
            market.epic.clone(),
            market.instrument_name.clone(),
            format!("{:?}", market.instrument_type),
            node_id.to_string(),
            self.exchange_name.clone(),
        );

        instrument.expiry = market.expiry.clone();
        instrument.high_limit_price = market.high_limit_price;
        instrument.low_limit_price = market.low_limit_price;
        instrument.market_status = market.market_status.clone();
        instrument.net_change = market.net_change;
        instrument.percentage_change = market.percentage_change;
        instrument.update_time = market.update_time.clone();
        instrument.bid = market.bid;
        instrument.offer = market.offer;

        instrument
    }

    /// Inserts a hierarchy node into the database
    async fn insert_hierarchy_node(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        node: &MarketHierarchyNode,
    ) -> Result<(), sqlx::Error> {
        tx.execute(
            sqlx::query(
                r#"
                INSERT INTO market_hierarchy_nodes 
                (id, name, parent_id, exchange, level, path, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT (id) DO UPDATE SET
                    name = EXCLUDED.name,
                    parent_id = EXCLUDED.parent_id,
                    exchange = EXCLUDED.exchange,
                    level = EXCLUDED.level,
                    path = EXCLUDED.path,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(&node.id)
            .bind(&node.name)
            .bind(&node.parent_id)
            .bind(&node.exchange)
            .bind(node.level)
            .bind(&node.path)
            .bind(node.created_at)
            .bind(node.updated_at),
        )
        .await?;

        Ok(())
    }

    /// Inserts a market instrument into the database
    async fn insert_market_instrument(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        instrument: &MarketInstrument,
    ) -> Result<(), sqlx::Error> {
        tx.execute(
            sqlx::query(
                r#"
                INSERT INTO market_instruments 
                (epic, instrument_name, instrument_type, node_id, exchange, expiry,
                 high_limit_price, low_limit_price, market_status, net_change, 
                 percentage_change, update_time, update_time_utc, bid, offer, 
                 created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                ON CONFLICT (epic) DO UPDATE SET
                    instrument_name = EXCLUDED.instrument_name,
                    instrument_type = EXCLUDED.instrument_type,
                    node_id = EXCLUDED.node_id,
                    exchange = EXCLUDED.exchange,
                    expiry = EXCLUDED.expiry,
                    high_limit_price = EXCLUDED.high_limit_price,
                    low_limit_price = EXCLUDED.low_limit_price,
                    market_status = EXCLUDED.market_status,
                    net_change = EXCLUDED.net_change,
                    percentage_change = EXCLUDED.percentage_change,
                    update_time = EXCLUDED.update_time,
                    update_time_utc = EXCLUDED.update_time_utc,
                    bid = EXCLUDED.bid,
                    offer = EXCLUDED.offer,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(&instrument.epic)
            .bind(&instrument.instrument_name)
            .bind(&instrument.instrument_type)
            .bind(&instrument.node_id)
            .bind(&instrument.exchange)
            .bind(&instrument.expiry)
            .bind(instrument.high_limit_price)
            .bind(instrument.low_limit_price)
            .bind(&instrument.market_status)
            .bind(instrument.net_change)
            .bind(instrument.percentage_change)
            .bind(&instrument.update_time)
            .bind(instrument.update_time_utc)
            .bind(instrument.bid)
            .bind(instrument.offer)
            .bind(instrument.created_at)
            .bind(instrument.updated_at),
        )
        .await?;

        Ok(())
    }

    /// Retrieves market hierarchy from the database
    pub async fn get_market_hierarchy(&self) -> Result<Vec<MarketHierarchyNode>, sqlx::Error> {
        let nodes = sqlx::query_as::<_, MarketHierarchyNode>(
            "SELECT * FROM market_hierarchy_nodes WHERE exchange = $1 ORDER BY level, name",
        )
        .bind(&self.exchange_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(nodes)
    }

    /// Retrieves market instruments for a specific node
    pub async fn get_instruments_by_node(
        &self,
        node_id: &str,
    ) -> Result<Vec<MarketInstrument>, sqlx::Error> {
        let instruments = sqlx::query_as::<_, MarketInstrument>(
            "SELECT * FROM market_instruments WHERE node_id = $1 AND exchange = $2 ORDER BY instrument_name",
        )
        .bind(node_id)
        .bind(&self.exchange_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(instruments)
    }

    /// Searches for instruments by name or epic
    pub async fn search_instruments(
        &self,
        search_term: &str,
    ) -> Result<Vec<MarketInstrument>, sqlx::Error> {
        let instruments = sqlx::query_as::<_, MarketInstrument>(
            r#"
            SELECT * FROM market_instruments 
            WHERE exchange = $1 
            AND (
                instrument_name ILIKE $2 
                OR epic ILIKE $2
                OR to_tsvector('english', instrument_name) @@ plainto_tsquery('english', $3)
            )
            ORDER BY instrument_name
            LIMIT 100
            "#,
        )
        .bind(&self.exchange_name)
        .bind(format!("%{search_term}%"))
        .bind(search_term)
        .fetch_all(&self.pool)
        .await?;

        Ok(instruments)
    }

    /// Gets statistics about the stored data
    pub async fn get_statistics(&self) -> Result<DatabaseStatistics, sqlx::Error> {
        let node_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM market_hierarchy_nodes WHERE exchange = $1")
                .bind(&self.exchange_name)
                .fetch_one(&self.pool)
                .await?;

        let instrument_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM market_instruments WHERE exchange = $1")
                .bind(&self.exchange_name)
                .fetch_one(&self.pool)
                .await?;

        let instrument_types: Vec<(String, i64)> = sqlx::query(
            "SELECT instrument_type, COUNT(*) as count FROM market_instruments WHERE exchange = $1 GROUP BY instrument_type ORDER BY count DESC",
        )
        .bind(&self.exchange_name)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| (row.get::<String, _>("instrument_type"), row.get::<i64, _>("count")))
        .collect();

        let max_depth: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(level), 0) FROM market_hierarchy_nodes WHERE exchange = $1",
        )
        .bind(&self.exchange_name)
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStatistics {
            exchange: self.exchange_name.clone(),
            node_count,
            instrument_count,
            instrument_types,
            max_hierarchy_depth: max_depth,
        })
    }
}

/// Statistics about the stored market data
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    pub exchange: String,
    pub node_count: i64,
    pub instrument_count: i64,
    pub instrument_types: Vec<(String, i64)>,
    pub max_hierarchy_depth: i32,
}

impl DatabaseStatistics {
    /// Prints a formatted summary of the statistics
    pub fn print_summary(&self) {
        info!("=== Market Database Statistics for {} ===", self.exchange);
        info!("Hierarchy nodes: {}", self.node_count);
        info!("Market instruments: {}", self.instrument_count);
        info!("Maximum hierarchy depth: {}", self.max_hierarchy_depth);
        info!("Instrument types:");
        for (instrument_type, count) in &self.instrument_types {
            info!("  {}: {}", instrument_type, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::models::market::InstrumentType;

    #[tokio::test]
    async fn test_convert_market_data_to_instrument() {
        let service = MarketDatabaseService::new(
            // This would be a real pool in actual tests
            PgPool::connect("postgresql://test")
                .await
                .unwrap_or_else(|_| panic!("Test requires a PostgreSQL connection")),
            "IG".to_string(),
        );

        let market_data = MarketData {
            epic: "IX.D.DAX.DAILY.IP".to_string(),
            instrument_name: "Germany 40".to_string(),
            instrument_type: InstrumentType::Indices,
            expiry: "DFB".to_string(),
            high_limit_price: Some(20000.0),
            low_limit_price: Some(5000.0),
            market_status: "TRADEABLE".to_string(),
            net_change: Some(100.5),
            percentage_change: Some(0.65),
            update_time: Some("2023-12-01T10:30:00".to_string()),
            update_time_utc: Some("2023-12-01T10:30:00Z".to_string()),
            bid: Some(15450.2),
            offer: Some(15451.8),
        };

        let instrument = service.convert_market_data_to_instrument(&market_data, "node_123");

        assert_eq!(instrument.epic, "IX.D.DAX.DAILY.IP");
        assert_eq!(instrument.instrument_name, "Germany 40");
        assert_eq!(instrument.instrument_type, "INDICES");
        assert_eq!(instrument.node_id, "node_123");
        assert_eq!(instrument.exchange, "IG");
        assert_eq!(instrument.expiry, "DFB");
        assert_eq!(instrument.high_limit_price, Some(20000.0));
        assert_eq!(instrument.bid, Some(15450.2));
        assert_eq!(instrument.offer, Some(15451.8));
    }
}
