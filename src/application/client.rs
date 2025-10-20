/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/

//! Simplified client for IG Markets API
//!
//! This module provides a clean, easy-to-use client that handles:
//! - Automatic authentication and token management
//! - Transparent OAuth token refresh
//! - Simple API for making requests
//!
//! # Example
//! ```ignore
//! use ig_client::client::Client;
//! use ig_client::config::Config;
//!
//! let config = Config::new();
//! let client = Client::new(config).await?;
//!
//! // Make requests - authentication is handled automatically
//! let accounts = client.get("/accounts").await?;
//! ```

use crate::application::auth::{Auth, Session};
use crate::application::config::Config;
use crate::application::rate_limiter::RateLimiter;
use crate::error::AppError;
use crate::model::http::make_http_request;
use crate::model::retry::RetryConfig;
use reqwest::{Client as HttpClient, Method, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

const USER_AGENT: &str = "ig-client/0.6.0";

/// Simplified client for IG Markets API with automatic authentication
///
/// This client handles all authentication complexity internally, including:
/// - Initial login
/// - OAuth token refresh
/// - Re-authentication when tokens expire
/// - Account switching
/// - Rate limiting for all API requests
pub struct Client {

}

impl Client {

}

impl Default for Client {
    fn default() -> Self {
        let config = Config::default();
        Self::new_lazy(config)
    }
}
