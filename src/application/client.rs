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
    auth: Arc<Auth>,
    http_client: HttpClient,
    config: Arc<Config>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

impl Client {
    /// Creates a new client and performs initial authentication
    ///
    /// # Arguments
    /// * `config` - Configuration containing credentials and API settings
    ///
    /// # Returns
    /// * `Ok(Client)` - Authenticated client ready to use
    /// * `Err(AppError)` - If authentication fails
    ///
    /// # Example
    /// ```ignore
    /// let config = Config::new();
    /// let client = Client::new(config).await?;
    /// ```
    pub async fn new(config: Config) -> Result<Self, AppError> {
        let config = Arc::new(config);
        let auth = Arc::new(Auth::new(config.clone()));

        // Perform initial login
        auth.login().await?;

        let http_client = HttpClient::builder().user_agent(USER_AGENT).build()?;
        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(&config.rate_limiter)));

        Ok(Self {
            auth,
            http_client,
            config,
            rate_limiter,
        })
    }

    /// Creates a new client without performing initial authentication
    ///
    /// Authentication will be performed automatically on the first request.
    ///
    /// # Arguments
    /// * `config` - Configuration containing credentials and API settings
    pub fn new_lazy(config: Config) -> Self {
        let config = Arc::new(config);
        let auth = Arc::new(Auth::new(config.clone()));

        let http_client = HttpClient::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(&config.rate_limiter)));

        Self {
            auth,
            http_client,
            config,
            rate_limiter,
        }
    }

    /// Makes a GET request to the IG Markets API
    ///
    /// # Arguments
    /// * `path` - API endpoint path (e.g., "/accounts")
    ///
    /// # Returns
    /// * `Ok(T)` - Deserialized response
    /// * `Err(AppError)` - If request fails
    ///
    /// # Example
    /// ```ignore
    /// let accounts: AccountsResponse = client.get("/accounts").await?;
    /// ```
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        self.request(Method::GET, path, None::<()>, None).await
    }

    /// Makes a POST request to the IG Markets API
    ///
    /// # Arguments
    /// * `path` - API endpoint path
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Returns
    /// * `Ok(T)` - Deserialized response
    /// * `Err(AppError)` - If request fails
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: B,
    ) -> Result<T, AppError> {
        self.request(Method::POST, path, Some(body), None).await
    }

    /// Makes a PUT request to the IG Markets API
    ///
    /// # Arguments
    /// * `path` - API endpoint path
    /// * `body` - Request body to serialize as JSON
    ///
    /// # Returns
    /// * `Ok(T)` - Deserialized response
    /// * `Err(AppError)` - If request fails
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: B,
    ) -> Result<T, AppError> {
        self.request(Method::PUT, path, Some(body), None).await
    }

    /// Makes a DELETE request to the IG Markets API
    ///
    /// # Arguments
    /// * `path` - API endpoint path
    ///
    /// # Returns
    /// * `Ok(T)` - Deserialized response
    /// * `Err(AppError)` - If request fails
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        self.request(Method::DELETE, path, None::<()>, None).await
    }

    /// Makes a request with custom API version
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `path` - API endpoint path
    /// * `body` - Optional request body
    /// * `version` - API version to use (overrides default)
    ///
    /// # Returns
    /// * `Ok(T)` - Deserialized response
    /// * `Err(AppError)` - If request fails
    pub async fn request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<B>,
        version: Option<&str>,
    ) -> Result<T, AppError> {
        // Try the request, automatically handling token refresh
        match self
            .request_internal(method.clone(), path, &body, version)
            .await
        {
            Ok(response) => self.parse_response(response).await,
            Err(AppError::OAuthTokenExpired) => {
                // Token expired, refresh and retry
                warn!("OAuth token expired, refreshing and retrying");
                self.auth.refresh_token().await?;

                let response = self.request_internal(method, path, &body, version).await?;
                self.parse_response(response).await
            }
            Err(e) => Err(e),
        }
    }

    /// Internal method to make HTTP requests using the common HTTP utility
    async fn request_internal<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &Option<B>,
        version: Option<&str>,
    ) -> Result<Response, AppError> {
        // Get current session (automatically refreshes if needed)
        let session = self.auth.get_session().await?;

        // Build URL
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            let path = path.trim_start_matches('/');
            format!("{}/{}", self.config.rest_api.base_url, path)
        };

        // Build headers vector - need to own strings for lifetime
        let api_key = self.config.credentials.api_key.clone();
        let version_owned = version
            .map(String::from)
            .unwrap_or_else(|| session.api_version.to_string());
        let auth_header_value;
        let account_id;
        let cst;
        let x_security_token;

        let mut headers = vec![
            ("X-IG-API-KEY", api_key.as_str()),
            ("Content-Type", "application/json; charset=UTF-8"),
            ("Accept", "application/json; charset=UTF-8"),
            ("Version", version_owned.as_str()),
        ];

        // Add authentication headers
        if let Some(oauth) = &session.oauth_token {
            // OAuth authentication (API v3)
            auth_header_value = format!("Bearer {}", oauth.access_token);
            account_id = session.account_id.clone();
            headers.push(("Authorization", auth_header_value.as_str()));
            headers.push(("IG-ACCOUNT-ID", account_id.as_str()));
        } else {
            // CST/X-SECURITY-TOKEN authentication (API v2)
            if let (Some(cst_val), Some(token_val)) = (&session.cst, &session.x_security_token) {
                cst = cst_val.clone();
                x_security_token = token_val.clone();
                headers.push(("CST", cst.as_str()));
                headers.push(("X-SECURITY-TOKEN", x_security_token.as_str()));
            }
        }

        // Use the common HTTP request function with infinite retries and 10 second delay
        make_http_request(
            &self.http_client,
            self.rate_limiter.clone(),
            method,
            &url,
            headers,
            body,
            RetryConfig::infinite(), // infinite retries with 10 second delay
        )
        .await
    }

    /// Parses a response into the desired type
    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, AppError> {
        Ok(response.json().await?)
    }

    /// Switches to a different trading account
    ///
    /// # Arguments
    /// * `account_id` - The account ID to switch to
    /// * `default_account` - Whether to set as default account
    ///
    /// # Returns
    /// * `Ok(())` - If account switch succeeds
    /// * `Err(AppError)` - If account switch fails
    pub async fn switch_account(
        &self,
        account_id: &str,
        default_account: Option<bool>,
    ) -> Result<(), AppError> {
        self.auth
            .switch_account(account_id, default_account)
            .await?;
        Ok(())
    }

    /// Gets the current session information
    ///
    /// # Returns
    /// * `Ok(Session)` - Current session with valid tokens
    /// * `Err(AppError)` - If unable to get valid session
    pub async fn get_session(&self) -> Result<Session, AppError> {
        self.auth.get_session().await
    }

    /// Logs out and clears the current session
    pub async fn logout(&self) -> Result<(), AppError> {
        self.auth.logout().await
    }

    /// Gets a reference to the underlying Auth instance
    pub fn auth(&self) -> &Auth {
        &self.auth
    }
}

impl Default for Client {
    fn default() -> Self {
        let config = Config::default();
        Self::new_lazy(config)
    }
}
