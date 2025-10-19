use async_trait::async_trait;
use once_cell::sync::Lazy;
use reqwest::{Client, Method, RequestBuilder, Response, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use crate::constants::USER_AGENT;
use crate::utils::rate_limiter::app_non_trading_limiter;
use crate::{config::Config, error::AppError, session::interface::IgSession};

// Global semaphore to limit concurrent API requests
// This ensures that we don't exceed rate limits by making too many
// concurrent requests
static API_SEMAPHORE: Lazy<Arc<Semaphore>> = Lazy::new(|| {
    Arc::new(Semaphore::new(3)) // Allow up to 3 concurrent requests
});

// Flag to indicate if we're in a rate-limited situation
static RATE_LIMITED: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

// Default retry configuration
const DEFAULT_MAX_RETRIES: u32 = 10; // Increase max retries to ensure all requests are processed
const DEFAULT_INITIAL_BACKOFF_MS: u64 = 1000; // 1 second
const DEFAULT_MAX_BACKOFF_MS: u64 = 60000; // 60 seconds max backoff
const DEFAULT_BACKOFF_FACTOR: f64 = 2.0; // Exponential backoff factor

/// Interface for the IG HTTP client
#[async_trait]
pub trait IgHttpClient: Send + Sync {
    /// Makes an HTTP request to the IG API
    async fn request<T, R>(
        &self,
        method: Method,
        path: &str,
        session: &IgSession,
        body: Option<&T>,
        version: &str,
    ) -> Result<R, AppError>
    where
        for<'de> R: DeserializeOwned + 'static,
        T: Serialize + Send + Sync + 'static;

    /// Makes an unauthenticated HTTP request (for login)
    async fn request_no_auth<T, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
        version: &str,
    ) -> Result<R, AppError>
    where
        for<'de> R: DeserializeOwned + 'static,
        T: Serialize + Send + Sync + 'static;
}

/// Implementation of the HTTP client for IG
pub struct IgHttpClientImpl {
    config: Arc<Config>,
    client: Client,
    max_retries: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    backoff_factor: f64,
}

impl IgHttpClientImpl {
    /// Creates a new instance of the HTTP client
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(config.rest_api.timeout))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff_ms: DEFAULT_INITIAL_BACKOFF_MS,
            max_backoff_ms: DEFAULT_MAX_BACKOFF_MS,
            backoff_factor: DEFAULT_BACKOFF_FACTOR,
        }
    }

    /// Configure retry behavior
    pub fn with_retry_config(
        mut self,
        max_retries: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
        backoff_factor: f64,
    ) -> Self {
        self.max_retries = max_retries;
        self.initial_backoff_ms = initial_backoff_ms;
        self.max_backoff_ms = max_backoff_ms;
        self.backoff_factor = backoff_factor;
        self
    }

    /// Calculate backoff duration for retry attempts with jitter
    fn calculate_backoff_duration(&self, retry_count: u32) -> Duration {
        use rand::Rng;
        let base_backoff_ms =
            (self.initial_backoff_ms as f64 * self.backoff_factor.powi(retry_count as i32)) as u64;
        let capped_backoff_ms = base_backoff_ms.min(self.max_backoff_ms);

        // Add jitter (±20%) to avoid thundering herd problem
        let jitter_factor = rand::rng().random_range(0.8..1.2);
        let jittered_backoff_ms = (capped_backoff_ms as f64 * jitter_factor) as u64;

        Duration::from_millis(jittered_backoff_ms)
    }

    /// Check if an error is retryable
    fn is_retryable_error(&self, error: &AppError) -> bool {
        match error {
            AppError::RateLimitExceeded => true,
            AppError::OAuthTokenExpired => true,
            AppError::Network(e) => {
                // Retry on connection errors, timeouts, and server errors
                e.is_timeout() || e.is_connect() || e.status().is_some_and(|s| s.is_server_error())
            }
            _ => false,
        }
    }

    /// Builds the complete URL for a request
    fn build_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.rest_api.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// Adds common headers to all requests
    fn add_common_headers(&self, builder: RequestBuilder, version: &str) -> RequestBuilder {
        let api_key = self.config.credentials.api_key.trim();
        debug!("Adding X-IG-API-KEY header (length: {})", api_key.len());
        if api_key.is_empty() {
            error!("API key is empty!");
        }
        builder
            .header("X-IG-API-KEY", api_key)
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("Accept", "application/json; charset=UTF-8")
            .header("Version", version)
    }

    /// Adds authentication headers to a request
    fn add_auth_headers(&self, builder: RequestBuilder, session: &IgSession) -> RequestBuilder {
        // Check if using OAuth (v3) or CST (v2) authentication
        if let Some(oauth_token) = &session.oauth_token {
            // Use OAuth Bearer token + IG-ACCOUNT-ID header
            // Per IG API docs: OAuth requires both Authorization and IG-ACCOUNT-ID headers
            debug!("Using OAuth authentication (Bearer token)");
            debug!(
                "   Access token: {}...",
                &oauth_token.access_token[..10.min(oauth_token.access_token.len())]
            );
            debug!("   Account ID: {}", session.account_id);
            builder
                .header(
                    "Authorization",
                    format!("Bearer {}", oauth_token.access_token),
                )
                .header("IG-ACCOUNT-ID", &session.account_id)
        } else {
            // Use CST and X-SECURITY-TOKEN (v2)
            debug!("Using CST authentication");
            debug!(
                "   CST length: {}, Token length: {}",
                session.cst.len(),
                session.token.len()
            );
            builder
                .header("CST", &session.cst)
                .header("X-SECURITY-TOKEN", &session.token)
        }
    }

    /// Processes the HTTP response and handles rate limiting centrally
    async fn process_response<R>(&self, response: Response) -> Result<R, AppError>
    where
        for<'de> R: DeserializeOwned + 'static,
    {
        let status = response.status();
        let url = response.url().to_string();

        // Handle rate limiting centrally
        if status == StatusCode::TOO_MANY_REQUESTS {
            self.handle_rate_limit(&url, "TOO_MANY_REQUESTS status code")
                .await;
            return Err(AppError::RateLimitExceeded);
        }

        match status {
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
                let body = response.text().await?;
                match serde_json::from_str::<R>(&body) {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        error!("Error deserializing response from {}: {}", url, e);
                        error!("Response body: {}", body);
                        Err(AppError::Json(e))
                    }
                }
            }
            StatusCode::UNAUTHORIZED => {
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read response body".to_string());
                error!("Unauthorized request to {}", url);
                error!("Response body: {}", body);

                // Check if this is an OAuth token expiration error
                if body.contains("error.security.oauth-token-invalid") {
                    debug!("Detected expired OAuth token");
                    Err(AppError::OAuthTokenExpired)
                } else {
                    Err(AppError::Unauthorized)
                }
            }
            StatusCode::NOT_FOUND => {
                error!("Resource not found at {}", url);
                Err(AppError::NotFound)
            }
            StatusCode::FORBIDDEN => {
                let body = response.text().await?;
                if body.contains("exceeded-api-key-allowance")
                    || body.contains("exceeded-account-allowance")
                {
                    self.handle_rate_limit(
                        &url,
                        "FORBIDDEN with exceeded-api-key-allowance or exceeded-account-allowance",
                    )
                    .await;
                    Err(AppError::RateLimitExceeded)
                } else {
                    error!("Forbidden access to {}: {}", url, body);
                    Err(AppError::Unauthorized)
                }
            }
            _ => {
                let body = response.text().await?;
                error!(
                    "Unexpected status code {} for request to {}: {}",
                    status, url, body
                );
                Err(AppError::Unexpected(status))
            }
        }
    }

    /// Helper method to handle rate limiting
    async fn handle_rate_limit(&self, url: &str, reason: &str) {
        // Set the rate limited flag
        RATE_LIMITED.store(true, Ordering::SeqCst);
        error!("Rate limit exceeded for request to {} ({})", url, reason);

        // Notify all rate limiters about the exceeded limit
        // This will cause them to enforce a mandatory cooldown period
        let non_trading_limiter = app_non_trading_limiter();
        non_trading_limiter.notify_rate_limit_exceeded().await;

        // Schedule a task to reset the flag after a delay
        // Increased from 30 to 60 seconds to give more time for rate limit to reset
        let rate_limited = RATE_LIMITED.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            rate_limited.store(false, Ordering::SeqCst);
            debug!("Rate limit flag reset after 60 second cooldown");
        });
    }
}

#[async_trait]
impl IgHttpClient for IgHttpClientImpl {
    async fn request<T, R>(
        &self,
        method: Method,
        path: &str,
        session: &IgSession,
        body: Option<&T>,
        version: &str,
    ) -> Result<R, AppError>
    where
        for<'de> R: DeserializeOwned + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        let url = self.build_url(path);
        let method_str = method.as_str().to_string(); // Store method as string for logging
        debug!("Making {} request to {}", method_str, url);

        let mut retry_count = 0;

        // Retry loop
        loop {
            // Check if we should retry
            if retry_count > 0 {
                if retry_count > self.max_retries {
                    warn!(
                        "Max retries ({}) exceeded for {} request to {}",
                        self.max_retries, method_str, url
                    );
                    break; // Exit the loop and try one last time without retrying
                }

                // Calculate backoff duration
                let backoff = self.calculate_backoff_duration(retry_count - 1);
                debug!(
                    "Retry attempt {} for {} request to {}. Waiting for {:?} before retrying",
                    retry_count, method_str, url, backoff
                );
                tokio::time::sleep(backoff).await;
            }

            // Check if we're currently rate limited
            if RATE_LIMITED.load(Ordering::SeqCst) {
                warn!("System is currently rate limited. Adding extra delay before request.");
                // Add a longer extra delay if we're in a rate-limited situation
                // Use retry count to increase delay for subsequent retries
                let rate_limit_delay = 2000 + (retry_count * 1000) as u64;
                tokio::time::sleep(tokio::time::Duration::from_millis(rate_limit_delay)).await;
            }

            // Acquire a permit from the semaphore to limit concurrent requests
            // This ensures we don't overwhelm the API with too many concurrent requests
            let permit = API_SEMAPHORE.acquire().await.unwrap();
            debug!(
                "Acquired API semaphore permit for {} request to {}",
                method_str, url
            );

            // Respect rate limits before making the request
            // This will handle the actual rate limiting based on request history
            match session.respect_rate_limit().await {
                Ok(()) => {}
                Err(e) => {
                    drop(permit);
                    if self.is_retryable_error(&e) {
                        retry_count += 1;
                        continue;
                    }
                    return Err(e);
                }
            }

            let mut builder = self.client.request(method.clone(), &url);
            builder = self.add_common_headers(builder, version);
            builder = self.add_auth_headers(builder, session);

            if let Some(data) = body {
                builder = builder.json(data);
            }

            // Send the request
            let response_result = builder.send().await;

            // Check for network errors
            let response = match response_result {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Network error for {} request to {}: {}", method_str, url, e);
                    // Release the permit before continuing
                    drop(permit);

                    // Check if we should retry
                    let app_error = AppError::Network(e);
                    if self.is_retryable_error(&app_error) {
                        retry_count += 1;
                        continue;
                    }
                    return Err(app_error);
                }
            };

            // Process the response - rate limiting is handled inside process_response
            let result = self.process_response::<R>(response).await;

            // If the request was successful, refresh token timer and reset rate limited flag
            if result.is_ok() {
                // Refresh token timer to extend token validity
                session.refresh_token_timer();

                if RATE_LIMITED.load(Ordering::SeqCst) {
                    RATE_LIMITED.store(false, Ordering::SeqCst);
                    debug!("Rate limit flag reset after successful request to {}", url);
                }
            }

            // Release the permit (this happens automatically when permit goes out of scope,
            // but we do it explicitly for clarity)
            drop(permit);

            // Handle the result
            match &result {
                Err(e) if self.is_retryable_error(e) => {
                    retry_count += 1;
                    continue;
                }
                _ => return result,
            }
        }

        // Final attempt without retrying
        info!(
            "Making final attempt for {} request to {} after max retries",
            method_str, url
        );

        // Acquire a permit from the semaphore
        let permit = API_SEMAPHORE.acquire().await.unwrap();

        // Respect rate limits
        session.respect_rate_limit().await?;

        let mut builder = self.client.request(method, &url);
        builder = self.add_common_headers(builder, version);
        builder = self.add_auth_headers(builder, session);

        if let Some(data) = body {
            builder = builder.json(data);
        }

        let response = builder.send().await?;
        let result = self.process_response::<R>(response).await;

        // If the final attempt was successful, refresh token timer
        if result.is_ok() {
            session.refresh_token_timer();
        }

        drop(permit);
        result
    }

    async fn request_no_auth<T, R>(
        &self,
        method: Method,
        path: &str,
        body: Option<&T>,
        version: &str,
    ) -> Result<R, AppError>
    where
        for<'de> R: DeserializeOwned + 'static,
        T: Serialize + Send + Sync + 'static,
    {
        let url = self.build_url(path);
        let method_str = method.as_str().to_string(); // Store method as string for logging
        info!("Making unauthenticated {} request to {}", method_str, url);

        let mut retry_count = 0;

        // Retry loop
        loop {
            // Check if we should retry
            if retry_count > 0 {
                if retry_count > self.max_retries {
                    warn!(
                        "Max retries ({}) exceeded for unauthenticated {} request to {}",
                        self.max_retries, method_str, url
                    );
                    break; // Exit the loop and try one last time without retrying
                }

                // Calculate backoff duration
                let backoff = self.calculate_backoff_duration(retry_count - 1);
                debug!(
                    "Retry attempt {} for unauthenticated {} request to {}. Waiting for {:?} before retrying",
                    retry_count, method_str, url, backoff
                );
                tokio::time::sleep(backoff).await;
            }

            // Check if we're currently rate limited
            if RATE_LIMITED.load(Ordering::SeqCst) {
                warn!(
                    "System is currently rate limited. Adding extra delay before unauthenticated request."
                );
                // Add a longer extra delay if we're in a rate-limited situation
                // Use retry count to increase delay for subsequent retries
                let rate_limit_delay = 1000 + (retry_count * 500) as u64;
                tokio::time::sleep(tokio::time::Duration::from_millis(rate_limit_delay)).await;
            }

            // Acquire a permit from the semaphore to limit concurrent requests
            let permit = API_SEMAPHORE.acquire().await.unwrap();
            debug!(
                "Acquired API semaphore permit for unauthenticated {} request to {}",
                method_str, url
            );

            // Use the global app rate limiter for unauthenticated requests
            // This is thread-safe and can be called from multiple threads concurrently
            let limiter = app_non_trading_limiter();
            limiter.wait().await;

            let mut builder = self.client.request(method.clone(), &url);
            builder = self.add_common_headers(builder, version);

            if let Some(data) = body {
                builder = builder.json(data);
            }

            // Send the request
            let response_result = builder.send().await;

            // Check for network errors
            let response = match response_result {
                Ok(resp) => resp,
                Err(e) => {
                    error!(
                        "Network error for unauthenticated {} request to {}: {}",
                        method_str, url, e
                    );
                    // Release the permit before continuing
                    drop(permit);

                    // Check if we should retry
                    let app_error = AppError::Network(e);
                    if self.is_retryable_error(&app_error) {
                        retry_count += 1;
                        continue;
                    }
                    return Err(app_error);
                }
            };

            // Process the response - rate limiting is handled inside process_response
            let result = self.process_response::<R>(response).await;

            // If the request was successful, reset the rate limited flag
            if result.is_ok() && RATE_LIMITED.load(Ordering::SeqCst) {
                RATE_LIMITED.store(false, Ordering::SeqCst);
                info!(
                    "Rate limit flag reset after successful unauthenticated request to {}",
                    url
                );
            }

            // Release the permit
            drop(permit);

            // Handle the result
            match &result {
                Err(e) if self.is_retryable_error(e) => {
                    retry_count += 1;
                    continue;
                }
                _ => return result,
            }
        }

        // Final attempt without retrying
        info!(
            "Making final attempt for unauthenticated {} request to {} after max retries",
            method_str, url
        );

        // Acquire a permit from the semaphore
        let permit = API_SEMAPHORE.acquire().await.unwrap();

        // Use the global app rate limiter
        let limiter = app_non_trading_limiter();
        limiter.wait().await;

        let mut builder = self.client.request(method, &url);
        builder = self.add_common_headers(builder, version);

        if let Some(data) = body {
            builder = builder.json(data);
        }

        let response = builder.send().await?;
        let result = self.process_response::<R>(response).await;

        drop(permit);
        result
    }
}
