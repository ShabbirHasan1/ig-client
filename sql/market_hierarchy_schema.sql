-- Market Hierarchy Database Schema for IG Markets
-- This script creates the necessary tables and indexes for storing market hierarchy data

-- Create market_hierarchy_nodes table
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

-- Create market_instruments table
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
);

-- Indexes for market_hierarchy_nodes table
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_parent_id ON market_hierarchy_nodes(parent_id);
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_exchange ON market_hierarchy_nodes(exchange);
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_level ON market_hierarchy_nodes(level);
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_path ON market_hierarchy_nodes USING gin(to_tsvector('english', path));
CREATE INDEX IF NOT EXISTS idx_market_hierarchy_name ON market_hierarchy_nodes USING gin(to_tsvector('english', name));

-- Indexes for market_instruments table
CREATE INDEX IF NOT EXISTS idx_market_instruments_node_id ON market_instruments(node_id);
CREATE INDEX IF NOT EXISTS idx_market_instruments_exchange ON market_instruments(exchange);
CREATE INDEX IF NOT EXISTS idx_market_instruments_type ON market_instruments(instrument_type);
CREATE INDEX IF NOT EXISTS idx_market_instruments_status ON market_instruments(market_status);
CREATE INDEX IF NOT EXISTS idx_market_instruments_name ON market_instruments USING gin(to_tsvector('english', instrument_name));
CREATE INDEX IF NOT EXISTS idx_market_instruments_epic ON market_instruments(epic);
CREATE INDEX IF NOT EXISTS idx_market_instruments_expiry ON market_instruments(expiry);

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

-- Create some useful views for querying

-- View to get the complete hierarchy with instrument counts
CREATE OR REPLACE VIEW market_hierarchy_with_counts AS
SELECT 
    n.id,
    n.name,
    n.parent_id,
    n.exchange,
    n.level,
    n.path,
    n.created_at,
    n.updated_at,
    COUNT(i.epic) as instrument_count,
    COUNT(DISTINCT i.instrument_type) as instrument_type_count
FROM market_hierarchy_nodes n
LEFT JOIN market_instruments i ON n.id = i.node_id
GROUP BY n.id, n.name, n.parent_id, n.exchange, n.level, n.path, n.created_at, n.updated_at
ORDER BY n.level, n.name;

-- View to get popular instruments (those with recent price updates)
CREATE OR REPLACE VIEW popular_instruments AS
SELECT 
    i.*,
    n.name as node_name,
    n.path as node_path
FROM market_instruments i
JOIN market_hierarchy_nodes n ON i.node_id = n.id
WHERE i.update_time_utc IS NOT NULL
  AND i.update_time_utc > NOW() - INTERVAL '1 day'
  AND i.market_status = 'TRADEABLE'
ORDER BY i.update_time_utc DESC;

-- View to get instrument statistics by type
CREATE OR REPLACE VIEW instrument_type_statistics AS
SELECT 
    exchange,
    instrument_type,
    COUNT(*) as total_count,
    COUNT(CASE WHEN market_status = 'TRADEABLE' THEN 1 END) as tradeable_count,
    COUNT(CASE WHEN bid IS NOT NULL AND offer IS NOT NULL THEN 1 END) as with_prices_count,
    AVG(CASE WHEN percentage_change IS NOT NULL THEN percentage_change END) as avg_percentage_change,
    MIN(update_time_utc) as oldest_update,
    MAX(update_time_utc) as newest_update
FROM market_instruments
GROUP BY exchange, instrument_type
ORDER BY exchange, total_count DESC;

-- Comments for documentation
COMMENT ON TABLE market_hierarchy_nodes IS 'Stores the hierarchical structure of market categories and subcategories';
COMMENT ON TABLE market_instruments IS 'Stores individual market instruments with their current market data';
COMMENT ON VIEW market_hierarchy_with_counts IS 'Provides hierarchy nodes with counts of instruments in each node';
COMMENT ON VIEW popular_instruments IS 'Shows recently updated tradeable instruments';
COMMENT ON VIEW instrument_type_statistics IS 'Provides statistics about instruments grouped by type and exchange';

-- Grant permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON market_hierarchy_nodes TO your_app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON market_instruments TO your_app_user;
-- GRANT SELECT ON market_hierarchy_with_counts TO your_app_user;
-- GRANT SELECT ON popular_instruments TO your_app_user;
-- GRANT SELECT ON instrument_type_statistics TO your_app_user;
