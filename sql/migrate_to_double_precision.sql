-- Migration script to change DECIMAL columns to DOUBLE PRECISION
-- This script updates the existing market_instruments table to use DOUBLE PRECISION instead of DECIMAL

-- Drop the existing table and recreate it with DOUBLE PRECISION
-- Note: This will lose existing data. In production, you might want to backup data first.

BEGIN;

-- Drop existing table
DROP TABLE IF EXISTS market_instruments CASCADE;

-- Recreate table with DOUBLE PRECISION
CREATE TABLE market_instruments (
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

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_market_instruments_node_id ON market_instruments(node_id);
CREATE INDEX IF NOT EXISTS idx_market_instruments_exchange ON market_instruments(exchange);
CREATE INDEX IF NOT EXISTS idx_market_instruments_type ON market_instruments(instrument_type);
CREATE INDEX IF NOT EXISTS idx_market_instruments_status ON market_instruments(market_status);
CREATE INDEX IF NOT EXISTS idx_market_instruments_name ON market_instruments USING gin(to_tsvector('english', instrument_name));
CREATE INDEX IF NOT EXISTS idx_market_instruments_epic ON market_instruments(epic);
CREATE INDEX IF NOT EXISTS idx_market_instruments_expiry ON market_instruments(expiry);

-- Recreate trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_market_instruments_updated_at 
    BEFORE UPDATE ON market_instruments 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_market_hierarchy_nodes_updated_at 
    BEFORE UPDATE ON market_hierarchy_nodes 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMIT;
