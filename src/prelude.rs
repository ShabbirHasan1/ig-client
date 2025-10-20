/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/10/25
******************************************************************************/

//! Prelude module for convenient imports
//!
//! This module re-exports commonly used types, traits, and functions
//! to make it easier to use the library.
//!
//! # Example
//! ```rust,ignore
//! use ig_client::prelude::*;
//!
//! let client = Client::default();
//! let markets = client.search_markets("EUR").await?;
//! ```

// Core client
pub use crate::application::client::Client;

// HTTP client
pub use crate::model::http::HttpClient;

// Authentication
pub use crate::application::auth::{Auth, Session};

// Configuration
pub use crate::application::config::{
    Config,
    Credentials,
    RestApiConfig,
    WebSocketConfig,
    RateLimiterConfig,
};

// Rate limiter
pub use crate::application::rate_limiter::RateLimiter;

// Service interfaces
pub use crate::application::interfaces::market::MarketService;

// Error handling
pub use crate::error::AppError;

// Common presentation models
pub use crate::presentation::market::{
    MarketSearchResult,
    MarketDetails,
    MarketData,
    MarketNavigationResponse,
    HistoricalPricesResponse,
};

pub use crate::presentation::account::{
    AccountInfo,
    Account,
    AccountBalance,
    AccountActivity,
    Activity,
};

pub use crate::presentation::trade::{
    TradeData,
    TradeFields,
    OpenPositionUpdate,
    WorkingOrderUpdate,
};

// Request models
pub use crate::model::requests::{
    RecentPricesRequest,
};

// Response models
pub use crate::model::responses::{
    DBEntryResponse,
    MultipleMarketDetailsResponse,
};

pub use crate::utils::*;

// Re-export commonly used external types
pub use async_trait::async_trait;
pub use serde::{Deserialize, Serialize};

// Result type alias for convenience
pub type IgResult<T> = Result<T, AppError>;
