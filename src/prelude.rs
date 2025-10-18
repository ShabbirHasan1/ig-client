/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/8/25
******************************************************************************/

//! # IG Client Prelude
//!
//! This module provides a convenient way to import the most commonly used types and traits
//! from the IG Client library. By importing this prelude, you get access to all the essential
//! components needed for most IG Markets API interactions.
//!
//! ## Usage
//!
//! ```rust
//! use ig_client::prelude::*;
//!
//! // Now you have access to all the commonly used types and traits
//! let config = Config::new();
//! let auth = IgAuth::new(&config);
//! // ... etc
//! ```

// ============================================================================
// CORE CONFIGURATION AND SETUP
// ============================================================================

/// Configuration for the IG Markets API client
pub use crate::config::Config;

/// Library version information
pub use crate::{VERSION, version};

// ============================================================================
// ERROR HANDLING
// ============================================================================

/// Main error type for the library
pub use crate::error::AppError;

// ============================================================================
// AUTHENTICATION AND SESSION MANAGEMENT
// ============================================================================

/// Authentication handler for IG Markets API
pub use crate::session::auth::IgAuth;

/// Authentication trait
pub use crate::session::interface::{IgAuthenticator, IgSession};

// ============================================================================
// CORE SERVICES (TRAITS)
// ============================================================================

/// Account service trait for account operations
pub use crate::application::*;

// ============================================================================
// TRANSPORT AND HTTP CLIENT
// ============================================================================

/// HTTP client trait
pub use crate::transport::http_client::IgHttpClient;

/// HTTP client implementation
pub use crate::transport::http_client::IgHttpClientImpl;

// ============================================================================
// PRESENTATION LAYER
// ============================================================================

/// Presentation layer types for UI and data display
pub use crate::presentation::{
    AccountData, ChartData, InstrumentType as PresentationInstrumentType, MarketFields,
    MarketState, PresentationMarketData, PriceData, TradeData,
};

/// Market hierarchy building functions
pub use crate::presentation::{build_market_hierarchy, extract_markets_from_hierarchy};

/// Serialization utilities
pub use crate::presentation::serialization::{
    option_string_empty_as_none, string_as_bool_opt, string_as_float_opt,
};

// ============================================================================
// UTILITIES
// ============================================================================

/// Rate limiting utilities
pub use crate::utils::*;

// ============================================================================
// STORAGE (OPTIONAL)
// ============================================================================

/// Database configuration (optional feature)
pub use crate::storage::config::DatabaseConfig;

/// Market database service (optional feature)
pub use crate::storage::market_database::MarketDatabaseService;

/// Database utilities (optional feature)
pub use crate::storage::utils::{create_connection_pool, create_database_config_from_env};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Global constants
pub use crate::constants::*;

// ============================================================================
// RE-EXPORTS FROM EXTERNAL CRATES
// ============================================================================

/// Re-export commonly used external types
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub use tokio;
pub use tracing::{debug, error, info, warn};

/// Re-export chrono for date/time handling
pub use chrono::{DateTime, Utc};

/// Re-export reqwest for HTTP operations (if needed for custom implementations)
pub use reqwest::Method;
