# Market Hierarchy Database Storage

This document explains how to use the market hierarchy database storage functionality to persist IG Markets data in PostgreSQL.

## Overview

The market hierarchy database storage system allows you to:

1. **Store market hierarchy data** from IG Markets in a PostgreSQL database
2. **Search and query** market instruments efficiently
3. **Track changes** over time with automatic timestamps
4. **Maintain data integrity** with proper foreign key relationships

## Database Schema

The system uses two main tables:

### `market_hierarchy_nodes`
Stores the hierarchical structure of market categories:
- `id`: Unique identifier for the node
- `name`: Human-readable name
- `parent_id`: Reference to parent node (NULL for root nodes)
- `exchange`: Exchange name (e.g., "IG")
- `level`: Depth in hierarchy (0 for root)
- `path`: Full path from root (e.g., "/Indices/Europe/Germany")
- `created_at`, `updated_at`: Automatic timestamps

### `market_instruments`
Stores individual market instruments:
- `epic`: Unique market identifier
- `instrument_name`: Human-readable instrument name
- `instrument_type`: Type (SHARES, INDICES, CURRENCIES, etc.)
- `node_id`: Reference to hierarchy node
- `exchange`: Exchange name
- Market data fields: `bid`, `offer`, `market_status`, etc.
- `created_at`, `updated_at`: Automatic timestamps

## Setup

### 1. Database Setup

First, create your PostgreSQL database and run the schema:

```bash
# Create database (if needed)
createdb ig_markets

# Run the schema
psql ig_markets < sql/market_hierarchy_schema.sql
```

### 2. Environment Variables

Set the required environment variables:

```bash
export DATABASE_URL="postgresql://username:password@localhost/ig_markets"
export DATABASE_MAX_CONNECTIONS="10"  # Optional, defaults to 10

# Your IG credentials (as usual)
export IG_USERNAME="your_username"
export IG_PASSWORD="your_password"
export IG_API_KEY="your_api_key"
export IG_ACCOUNT_ID="your_account_id"
export IG_ENVIRONMENT="demo"  # or "production"
```

### 3. Run the Example

```bash
cd examples/market
cargo run --bin market_hierarchy_to_db
```

## Usage Examples

### Basic Usage

```rust
use ig_client::storage::market_database::MarketDatabaseService;
use ig_client::storage::utils::{create_connection_pool, create_database_config_from_env};

// Create database connection
let db_config = create_database_config_from_env()?;
let pool = create_connection_pool(&db_config).await?;

// Create service
let db_service = MarketDatabaseService::new(pool, "IG".to_string());

// Initialize tables
db_service.initialize_database().await?;

// Store hierarchy (assuming you have hierarchy data)
db_service.store_market_hierarchy(&hierarchy).await?;
```

### Searching Instruments

```rust
// Search for instruments by name
let instruments = db_service.search_instruments("Germany").await?;

// Get instruments in a specific node
let instruments = db_service.get_instruments_by_node("node_id").await?;

// Get statistics
let stats = db_service.get_statistics().await?;
stats.print_summary();
```

### Using SQL Views

The schema includes several useful views:

```sql
-- Get hierarchy with instrument counts
SELECT * FROM market_hierarchy_with_counts WHERE exchange = 'IG';

-- Get recently updated instruments
SELECT * FROM popular_instruments LIMIT 10;

-- Get statistics by instrument type
SELECT * FROM instrument_type_statistics WHERE exchange = 'IG';
```

## Features

### Efficient Storage
- **Normalized schema** prevents data duplication
- **Proper indexing** for fast queries
- **Full-text search** capabilities for instrument names

### Data Integrity
- **Foreign key constraints** maintain referential integrity
- **Automatic timestamps** track when data was created/updated
- **Upsert operations** handle duplicate data gracefully

### Search Capabilities
- **Text search** using PostgreSQL's full-text search
- **Hierarchical queries** to find instruments in specific categories
- **Pattern matching** for epic codes and names

### Performance Optimizations
- **GIN indexes** for full-text search
- **B-tree indexes** for common query patterns
- **Connection pooling** for efficient database access

## Database Queries

### Common Queries

```sql
-- Find all German indices
SELECT i.* FROM market_instruments i
JOIN market_hierarchy_nodes n ON i.node_id = n.id
WHERE n.path LIKE '%Germany%' AND i.instrument_type = 'INDICES';

-- Get hierarchy structure
WITH RECURSIVE hierarchy AS (
  SELECT id, name, parent_id, path, 0 as depth
  FROM market_hierarchy_nodes 
  WHERE parent_id IS NULL AND exchange = 'IG'
  
  UNION ALL
  
  SELECT n.id, n.name, n.parent_id, n.path, h.depth + 1
  FROM market_hierarchy_nodes n
  JOIN hierarchy h ON n.parent_id = h.id
)
SELECT * FROM hierarchy ORDER BY depth, name;

-- Get most active instruments (by recent updates)
SELECT instrument_name, epic, market_status, percentage_change, update_time_utc
FROM market_instruments 
WHERE exchange = 'IG' 
  AND update_time_utc > NOW() - INTERVAL '1 hour'
  AND market_status = 'TRADEABLE'
ORDER BY update_time_utc DESC;
```

### Performance Monitoring

```sql
-- Check table sizes
SELECT 
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE tablename IN ('market_hierarchy_nodes', 'market_instruments');

-- Check index usage
SELECT 
  schemaname,
  tablename,
  indexname,
  idx_scan,
  idx_tup_read,
  idx_tup_fetch
FROM pg_stat_user_indexes 
WHERE tablename IN ('market_hierarchy_nodes', 'market_instruments');
```

## Troubleshooting

### Common Issues

1. **Connection Issues**
   - Verify `DATABASE_URL` is correct
   - Check PostgreSQL is running
   - Ensure database exists

2. **Permission Issues**
   - Grant necessary permissions to your database user
   - Check schema creation permissions

3. **Performance Issues**
   - Monitor index usage
   - Consider increasing `DATABASE_MAX_CONNECTIONS`
   - Check query execution plans

### Logging

The system uses `tracing` for logging. Enable debug logging to see detailed database operations:

```bash
RUST_LOG=debug cargo run --bin market_hierarchy_to_db
```

## Migration and Maintenance

### Updating Schema

When updating the schema, create migration scripts:

```sql
-- Example migration
ALTER TABLE market_instruments ADD COLUMN new_field VARCHAR(100);
CREATE INDEX idx_market_instruments_new_field ON market_instruments(new_field);
```

### Data Cleanup

```sql
-- Remove old data (example: older than 30 days)
DELETE FROM market_instruments 
WHERE created_at < NOW() - INTERVAL '30 days';

-- Vacuum tables for performance
VACUUM ANALYZE market_hierarchy_nodes;
VACUUM ANALYZE market_instruments;
```

## Integration with Other Systems

The stored data can be easily integrated with:

- **Analytics tools** (Grafana, Tableau)
- **Trading algorithms** (direct SQL queries)
- **Reporting systems** (using the provided views)
- **Data pipelines** (ETL processes)

## Security Considerations

- Use connection pooling to limit database connections
- Implement proper user permissions
- Consider encryption for sensitive data
- Regular backups of the database
- Monitor for unusual query patterns
