use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a market hierarchy node in the database
/// This structure is optimized for PostgreSQL storage with proper indexing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketHierarchyNode {
    /// Unique identifier for the node
    pub id: String,
    /// Human-readable name of the node
    pub name: String,
    /// Parent node ID (NULL for root nodes)
    pub parent_id: Option<String>,
    /// Exchange name (e.g., "IG")
    pub exchange: String,
    /// Depth level in the hierarchy (0 for root nodes)
    pub level: i32,
    /// Full path from root to this node (e.g., "/Indices/Europe/Germany")
    pub path: String,
    /// Timestamp when this record was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when this record was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents a market instrument in the database
/// This structure is optimized for PostgreSQL storage with proper indexing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MarketInstrument {
    /// Unique identifier for the market (epic)
    pub epic: String,
    /// Human-readable name of the instrument
    pub instrument_name: String,
    /// Type of the instrument (e.g., "SHARES", "INDICES", "CURRENCIES")
    pub instrument_type: String,
    /// Node ID where this instrument belongs
    pub node_id: String,
    /// Exchange name (e.g., "IG")
    pub exchange: String,
    /// Expiry date of the instrument (empty string for perpetual instruments)
    pub expiry: String,
    /// Upper price limit for the market
    pub high_limit_price: Option<f64>,
    /// Lower price limit for the market
    pub low_limit_price: Option<f64>,
    /// Current status of the market
    pub market_status: String,
    /// Net change in price since previous close
    pub net_change: Option<f64>,
    /// Percentage change in price since previous close
    pub percentage_change: Option<f64>,
    /// Time of the last price update
    pub update_time: Option<String>,
    /// Time of the last price update in UTC
    pub update_time_utc: Option<DateTime<Utc>>,
    /// Current bid price
    pub bid: Option<f64>,
    /// Current offer/ask price
    pub offer: Option<f64>,
    /// Timestamp when this record was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when this record was last updated
    pub updated_at: DateTime<Utc>,
}

/// SQL DDL statements for creating the required tables
pub const CREATE_MARKET_HIERARCHY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS market_hierarchy_nodes (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    parent_id VARCHAR(255) REFERENCES market_hierarchy_nodes(id),
    exchange VARCHAR(50) NOT NULL,
    level INTEGER NOT NULL DEFAULT 0,
    path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_parent_id ON market_hierarchy_nodes(parent_id);
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_exchange ON market_hierarchy_nodes(exchange);
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_level ON market_hierarchy_nodes(level);
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_path ON market_hierarchy_nodes USING gin(to_tsvector('english', path));
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_name ON market_hierarchy_nodes USING gin(to_tsvector('english', name));
"#;

pub const CREATE_MARKET_INSTRUMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS market_instruments (
    epic VARCHAR(255) PRIMARY KEY,
    instrument_name VARCHAR(500) NOT NULL,
    instrument_type VARCHAR(100) NOT NULL,
    node_id VARCHAR(255) NOT NULL REFERENCES market_hierarchy_nodes(id),
    exchange VARCHAR(50) NOT NULL,
    expiry VARCHAR(50) NOT NULL DEFAULT '',
    high_limit_price DOUBLE PRECISION,
    low_limit_price DOUBLE PRECISION,
    market_status VARCHAR(50) NOT NULL,
    net_change DOUBLE PRECISION,
    percentage_change DOUBLE PRECISION,
    update_time VARCHAR(50),
    update_time_utc TIMESTAMPTZ,
    bid DOUBLE PRECISION,
    offer DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_market_instruments_node_id ON market_instruments(node_id);
CREATE INDEX IF NOT EXISTS idx_market_instruments_exchange ON market_instruments(exchange);
CREATE INDEX IF NOT EXISTS idx_market_instruments_type ON market_instruments(instrument_type);
CREATE INDEX IF NOT EXISTS idx_market_instruments_status ON market_instruments(market_status);
CREATE INDEX IF NOT EXISTS idx_market_instruments_name ON market_instruments USING gin(to_tsvector('english', instrument_name));
CREATE INDEX IF NOT EXISTS idx_market_instruments_epic ON market_instruments(epic);
CREATE INDEX IF NOT EXISTS idx_market_instruments_expiry ON market_instruments(expiry);
"#;

/// Trigger to automatically update the updated_at timestamp
pub const CREATE_UPDATE_TIMESTAMP_TRIGGER: &str = r#"
-- Function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for both tables
DROP TRIGGER IF EXISTS update_market_hierarchy_nodes_updated_at ON market_hierarchy_nodes;
CREATE TRIGGER update_market_hierarchy_nodes_updated_at
    BEFORE UPDATE ON market_hierarchy_nodes
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_market_instruments_updated_at ON market_instruments;
CREATE TRIGGER update_market_instruments_updated_at
    BEFORE UPDATE ON market_instruments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
"#;

impl MarketHierarchyNode {
    /// Creates a new MarketHierarchyNode
    pub fn new(
        id: String,
        name: String,
        parent_id: Option<String>,
        exchange: String,
        level: i32,
        path: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            parent_id,
            exchange,
            level,
            path,
            created_at: now,
            updated_at: now,
        }
    }

    /// Builds the full path for a node based on its parent path
    pub fn build_path(parent_path: Option<&str>, node_name: &str) -> String {
        match parent_path {
            Some(parent) if !parent.is_empty() => format!("{parent}/{node_name}"),
            _ => format!("/{node_name}"),
        }
    }
}

impl MarketInstrument {
    /// Creates a new MarketInstrument
    pub fn new(
        epic: String,
        instrument_name: String,
        instrument_type: String,
        node_id: String,
        exchange: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            epic,
            instrument_name,
            instrument_type,
            node_id,
            exchange,
            expiry: String::new(),
            high_limit_price: None,
            low_limit_price: None,
            market_status: String::new(),
            net_change: None,
            percentage_change: None,
            update_time: None,
            update_time_utc: None,
            bid: None,
            offer: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Parses the update_time_utc from a string if available
    pub fn parse_update_time_utc(&mut self) {
        if let Some(ref time_str) = self.update_time {
            if let Ok(parsed_time) = DateTime::parse_from_rfc3339(time_str) {
                self.update_time_utc = Some(parsed_time.with_timezone(&Utc));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_path() {
        assert_eq!(MarketHierarchyNode::build_path(None, "Root"), "/Root");
        assert_eq!(
            MarketHierarchyNode::build_path(Some("/Root"), "Child"),
            "/Root/Child"
        );
        assert_eq!(
            MarketHierarchyNode::build_path(Some("/Root/Child"), "Grandchild"),
            "/Root/Child/Grandchild"
        );
    }

    #[test]
    fn test_market_hierarchy_node_creation() {
        let node = MarketHierarchyNode::new(
            "test_id".to_string(),
            "Test Node".to_string(),
            Some("parent_id".to_string()),
            "IG".to_string(),
            1,
            "/Test Node".to_string(),
        );

        assert_eq!(node.id, "test_id");
        assert_eq!(node.name, "Test Node");
        assert_eq!(node.parent_id, Some("parent_id".to_string()));
        assert_eq!(node.exchange, "IG");
        assert_eq!(node.level, 1);
        assert_eq!(node.path, "/Test Node");
    }

    #[test]
    fn test_market_instrument_creation() {
        let mut instrument = MarketInstrument::new(
            "IX.D.DAX.DAILY.IP".to_string(),
            "Germany 40".to_string(),
            "INDICES".to_string(),
            "node_123".to_string(),
            "IG".to_string(),
        );

        assert_eq!(instrument.epic, "IX.D.DAX.DAILY.IP");
        assert_eq!(instrument.instrument_name, "Germany 40");
        assert_eq!(instrument.instrument_type, "INDICES");
        assert_eq!(instrument.node_id, "node_123");
        assert_eq!(instrument.exchange, "IG");

        // Test update_time_utc parsing
        instrument.update_time = Some("2023-12-01T10:30:00Z".to_string());
        instrument.parse_update_time_utc();
        assert!(instrument.update_time_utc.is_some());
    }
}
