/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/10/25
******************************************************************************/

//! HTTP request utilities with rate limiting and automatic retry
//!
//! This module provides a common HTTP request function that handles:
//! - Rate limiting before each request
//! - Automatic retry on rate limit exceeded errors
//! - Consistent error handling
//! - Support for all HTTP methods

use crate::application::rate_limiter::RateLimiter;
use crate::error::AppError;
use crate::model::retry::RetryConfig;
use reqwest::{Client, Method, Response, StatusCode};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

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
            println!("Request arrived!");
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
