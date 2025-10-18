use pretty_simple_display::DisplaySimple;
use serde::{Deserialize, Serialize};

/// Configuration for database connections
#[derive(Debug, DisplaySimple, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// Maximum number of connections in the connection pool
    pub max_connections: u32,
}
