/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/10/25
******************************************************************************/

use crate::application::auth::{Auth, Session, WebsocketInfo};
use crate::application::config::Config;
use crate::application::rate_limiter::RateLimiter;
use crate::error::AppError;
use crate::model::retry::RetryConfig;
use reqwest::Client as HttpInternalClient;
use reqwest::{Client, Method, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

const USER_AGENT: &str = "ig-client/0.6.0";

/// Simplified client for IG Markets API with automatic authentication
///
/// This client handles all authentication complexity internally, including:
/// - Initial login
/// - OAuth token refresh
/// - Re-authentication when tokens expire
/// - Account switching
/// - Rate limiting for all API requests
pub struct HttpClient {
    auth: Arc<Auth>,
    http_client: HttpInternalClient,
    config: Arc<Config>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

impl HttpClient {
    /// Creates a new client and performs initial authentication
    ///
    /// # Arguments
    /// * `config` - Configuration containing credentials and API settings
    ///
    /// # Returns
    /// * `Ok(Client)` - Authenticated client ready to use
    /// * `Err(AppError)` - If authentication fails
    pub async fn new(config: Config) -> Result<Self, AppError> {
        let config = Arc::new(config);

        // Create HTTP client and rate limiter first
        let http_client = HttpInternalClient::builder()
            .user_agent(USER_AGENT)
            .build()?;
        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(&config.rate_limiter)));

        // Create Auth instance
        let auth = Arc::new(Auth::new(config.clone()));

        // Perform initial login
        auth.login().await?;

        Ok(Self {
            auth,
            http_client,
            config,
            rate_limiter,
        })
    }

    /// Creates a new client without performing initial authentication
    pub fn new_lazy(config: Config) -> Self {
        let config = Arc::new(config);

        // Create HTTP client and rate limiter first
        let http_client = HttpInternalClient::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");
        let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(&config.rate_limiter)));

        // Create Auth instance
        let auth = Arc::new(Auth::new(config.clone()));

        Self {
            auth,
            http_client,
            config,
            rate_limiter,
        }
    }

    pub async fn get_ws_info(&self) -> WebsocketInfo {
        self.auth.get_ws_info().await
    }

    /// Makes a GET request
    pub async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        version: Option<u8>,
    ) -> Result<T, AppError> {
        self.request(Method::GET, path, None::<()>, version).await
    }

    /// Makes a POST request
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: B,
        version: Option<u8>,
    ) -> Result<T, AppError> {
        self.request(Method::POST, path, Some(body), version).await
    }

    /// Makes a PUT request
    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: B,
        version: Option<u8>,
    ) -> Result<T, AppError> {
        self.request(Method::PUT, path, Some(body), version).await
    }

    /// Makes a DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        self.request(Method::DELETE, path, None::<()>, None).await
    }

    /// Makes a POST request with _method: DELETE header
    ///
    /// This is required by IG API for closing positions, as they don't support
    /// DELETE requests with a body. Instead, they use POST with a special header.
    ///
    /// # Arguments
    /// * `path` - API endpoint path
    /// * `body` - Request body to send
    /// * `version` - API version to use
    ///
    /// # Returns
    /// Deserialized response of type T
    pub async fn post_with_delete_method<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: B,
        version: Option<u8>,
    ) -> Result<T, AppError> {
        match self
            .request_internal_with_delete_method(path, &body, version)
            .await
        {
            Ok(response) => self.parse_response(response).await,
            Err(AppError::OAuthTokenExpired) => {
                warn!("OAuth token expired, refreshing and retrying");
                self.auth.refresh_token().await?;
                let response = self
                    .request_internal_with_delete_method(path, &body, version)
                    .await?;
                self.parse_response(response).await
            }
            Err(e) => Err(e),
        }
    }

    /// Makes a request with custom API version
    pub async fn request<B: Serialize, T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<B>,
        version: Option<u8>,
    ) -> Result<T, AppError> {
        match self
            .request_internal(method.clone(), path, &body, version)
            .await
        {
            Ok(response) => self.parse_response(response).await,
            Err(AppError::OAuthTokenExpired) => {
                warn!("OAuth token expired, refreshing and retrying");
                self.auth.refresh_token().await?;
                let response = self.request_internal(method, path, &body, version).await?;
                self.parse_response(response).await
            }
            Err(e) => Err(e),
        }
    }

    /// Internal method to make HTTP requests
    async fn request_internal<B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: &Option<B>,
        version: Option<u8>,
    ) -> Result<Response, AppError> {
        let session = self.auth.get_session().await?;

        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            let path = path.trim_start_matches('/');
            format!("{}/{}", self.config.rest_api.base_url, path)
        };

        let api_key = self.config.credentials.api_key.clone();
        let version_owned = version.unwrap_or(1).to_string();
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

        if let Some(oauth) = &session.oauth_token {
            auth_header_value = format!("Bearer {}", oauth.access_token);
            account_id = session.account_id.clone();
            headers.push(("Authorization", auth_header_value.as_str()));
            headers.push(("IG-ACCOUNT-ID", account_id.as_str()));
        } else if let (Some(cst_val), Some(token_val)) = (&session.cst, &session.x_security_token) {
            cst = cst_val.clone();
            x_security_token = token_val.clone();
            headers.push(("CST", cst.as_str()));
            headers.push(("X-SECURITY-TOKEN", x_security_token.as_str()));
        }

        make_http_request(
            &self.http_client,
            self.rate_limiter.clone(),
            method,
            &url,
            headers,
            body,
            RetryConfig::infinite(),
        )
        .await
    }

    /// Internal method to make POST requests with _method: DELETE header
    ///
    /// This is required by IG API for closing positions
    async fn request_internal_with_delete_method<B: Serialize>(
        &self,
        path: &str,
        body: &B,
        version: Option<u8>,
    ) -> Result<Response, AppError> {
        let session = self.auth.get_session().await?;

        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            let path = path.trim_start_matches('/');
            format!("{}/{}", self.config.rest_api.base_url, path)
        };

        let api_key = self.config.credentials.api_key.clone();
        let version_owned = version.unwrap_or(1).to_string();
        let auth_header_value;
        let account_id;
        let cst;
        let x_security_token;

        let mut headers = vec![
            ("X-IG-API-KEY", api_key.as_str()),
            ("Content-Type", "application/json; charset=UTF-8"),
            ("Accept", "application/json; charset=UTF-8"),
            ("Version", version_owned.as_str()),
            ("_method", "DELETE"), // Special header for IG API
        ];

        if let Some(oauth) = &session.oauth_token {
            auth_header_value = format!("Bearer {}", oauth.access_token);
            account_id = session.account_id.clone();
            headers.push(("Authorization", auth_header_value.as_str()));
            headers.push(("IG-ACCOUNT-ID", account_id.as_str()));
        } else if let (Some(cst_val), Some(token_val)) = (&session.cst, &session.x_security_token) {
            cst = cst_val.clone();
            x_security_token = token_val.clone();
            headers.push(("CST", cst.as_str()));
            headers.push(("X-SECURITY-TOKEN", x_security_token.as_str()));
        }

        make_http_request(
            &self.http_client,
            self.rate_limiter.clone(),
            Method::POST, // Always POST for this method
            &url,
            headers,
            &Some(body),
            RetryConfig::infinite(),
        )
        .await
    }

    /// Parses response
    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T, AppError> {
        Ok(response.json().await?)
    }

    /// Switches to a different trading account
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

    /// Gets the current session
    pub async fn get_session(&self) -> Result<Session, AppError> {
        self.auth.get_session().await
    }

    /// Logs out
    pub async fn logout(&self) -> Result<(), AppError> {
        self.auth.logout().await
    }

    /// Gets Auth reference
    pub fn auth(&self) -> &Auth {
        &self.auth
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        let config = Config::default();
        Self::new_lazy(config)
    }
}

/// Makes an HTTP request with automatic rate limiting and retry on rate limit errors
///
/// This function provides a centralized way to make HTTP requests to the IG Markets API
/// with built-in rate limiting and automatic retry logic.
///
/// # Arguments
///
/// * `client` - The HTTP client to use for the request
/// * `rate_limiter` - Shared rate limiter to control request rate
/// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
/// * `url` - Full URL to request
/// * `headers` - Vector of (header_name, header_value) tuples
/// * `body` - Optional request body (will be serialized to JSON)
/// * `retry_config` - Retry configuration (max retries and delay)
///
/// # Returns
///
/// * `Ok(Response)` - Successful HTTP response
/// * `Err(AppError)` - Error if request fails (excluding rate limit errors which are retried)
///
/// # Example
///
/// ```ignore
/// use ig_client::model::http::{make_http_request, RetryConfig};
/// use reqwest::{Client, Method};
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
///
/// let client = Client::new();
/// let rate_limiter = Arc::new(RwLock::new(RateLimiter::new(&config)));
/// let headers = vec![
///     ("X-IG-API-KEY", "your-api-key"),
///     ("Content-Type", "application/json"),
/// ];
///
/// // Infinite retries with 10 second delay (default)
/// let response = make_http_request(
///     &client,
///     rate_limiter.clone(),
///     Method::GET,
///     "https://demo-api.ig.com/gateway/deal/markets/EPIC",
///     headers.clone(),
///     &None::<()>,
///     RetryConfig::infinite(),
/// ).await?;
///
/// // Maximum 3 retries with default 10 second delay
/// let response = make_http_request(
///     &client,
///     rate_limiter.clone(),
///     Method::GET,
///     "https://demo-api.ig.com/gateway/deal/markets/EPIC",
///     headers.clone(),
///     &None::<()>,
///     RetryConfig::with_max_retries(3),
/// ).await?;
///
/// // Infinite retries with custom 5 second delay
/// let response = make_http_request(
///     &client,
///     rate_limiter.clone(),
///     Method::GET,
///     "https://demo-api.ig.com/gateway/deal/markets/EPIC",
///     headers.clone(),
///     &None::<()>,
///     RetryConfig::with_delay(5),
/// ).await?;
///
/// // Maximum 3 retries with custom 5 second delay
/// let response = make_http_request(
///     &client,
///     rate_limiter,
///     Method::GET,
///     "https://demo-api.ig.com/gateway/deal/markets/EPIC",
///     headers,
///     &None::<()>,
///     RetryConfig::with_max_retries_and_delay(3, 5),
/// ).await?;
/// ```
pub async fn make_http_request<B: Serialize>(
    client: &Client,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    method: Method,
    url: &str,
    headers: Vec<(&str, &str)>,
    body: &Option<B>,
    retry_config: RetryConfig,
) -> Result<Response, AppError> {
    let mut retry_count = 0;
    let max_retries = retry_config.max_retries();
    let delay_secs = retry_config.delay_secs();

    loop {
        // Wait for rate limiter before making request
        {
            let limiter = rate_limiter.read().await;
            limiter.wait().await;
        }

        debug!("{} {}", method, url);

        // Build request
        let mut request = client.request(method.clone(), url);

        // Add headers
        for (name, value) in &headers {
            request = request.header(*name, *value);
        }

        // Add body if present
        if let Some(b) = body {
            request = request.json(b);
        }

        // Send request
        let response = request.send().await?;
        let status = response.status();
        debug!("Response status: {}", status);

        if status.is_success() {
            return Ok(response);
        }

        match status {
            StatusCode::FORBIDDEN => {
                let body_text = response.text().await.unwrap_or_default();
                if body_text.contains("exceeded-api-key-allowance")
                    || body_text.contains("exceeded-account-allowance")
                    || body_text.contains("exceeded-account-trading-allowance")
                    || body_text.contains("exceeded-account-historical-data-allowance")
                {
                    retry_count += 1;

                    // Check if we've exceeded max retries (0 = infinite)
                    if max_retries > 0 && retry_count > max_retries {
                        error!(
                            "Rate limit exceeded after {} attempts. Max retries ({}) reached.",
                            retry_count - 1,
                            max_retries
                        );
                        return Err(AppError::RateLimitExceeded);
                    }

                    warn!(
                        "Rate limit exceeded (attempt {}): {}. Waiting {} seconds before retry...",
                        retry_count, body_text, delay_secs
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
                    continue; // Retry the request
                }
                error!("Forbidden: {}", body_text);
                return Err(AppError::Unexpected(status));
            }
            StatusCode::UNAUTHORIZED => {
                let body_text = response.text().await.unwrap_or_default();
                if body_text.contains("oauth-token-invalid") {
                    return Err(AppError::OAuthTokenExpired);
                }
                error!("Unauthorized: {}", body_text);
                return Err(AppError::Unauthorized);
            }
            _ => {
                let body = response.text().await.unwrap_or_default();
                error!("Request failed with status {}: {}", status, body);
                return Err(AppError::Unexpected(status));
            }
        }
    }
}
