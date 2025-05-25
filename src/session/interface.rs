use crate::config::Config;
use crate::error::{AppError, AuthError};
use crate::utils::rate_limiter::{
    RateLimitType, RateLimiter, RateLimiterStats, app_non_trading_limiter, create_rate_limiter,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

/// Session information for IG Markets API authentication
#[derive(Debug, Clone)]
pub struct IgSession {
    /// Client Session Token (CST) used for authentication
    pub cst: String,
    /// Security token used for authentication
    pub token: String,
    /// Account ID associated with the session
    pub account_id: String,
    /// Base URL for API requests
    pub base_url: String,
    /// Client ID for API requests
    pub client_id: String,
    /// Lightstreamer endpoint for API requests
    pub lightstreamer_endpoint: String,
    /// API key for API requests
    pub api_key: String,
    /// Rate limiter for controlling request rates
    pub(crate) rate_limiter: Option<Arc<RateLimiter>>,
    /// Flag to indicate if the session is being used in a concurrent context
    pub(crate) concurrent_mode: Arc<AtomicBool>,
}

impl IgSession {
    /// Creates a new session with the given credentials
    ///
    /// This is a simplified version for tests and basic usage.
    /// Uses default values for most fields and a default rate limiter.
    pub fn new(cst: String, token: String, account_id: String) -> Self {
        Self {
            base_url: String::new(),
            cst,
            token,
            client_id: String::new(),
            account_id,
            lightstreamer_endpoint: String::new(),
            api_key: String::new(),
            rate_limiter: Some(create_rate_limiter(
                RateLimitType::NonTradingAccount,
                Some(0.8),
            )),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a new session with the given parameters
    ///
    /// This creates a thread-safe session that can be shared across multiple threads.
    /// The rate limiter is wrapped in an Arc to ensure proper synchronization.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_config(
        base_url: String,
        cst: String,
        security_token: String,
        client_id: String,
        account_id: String,
        lightstreamer_endpoint: String,
        api_key: String,
        rate_limit_type: RateLimitType,
        rate_limit_safety_margin: f64,
    ) -> Self {
        // Create a rate limiter with the specified type and safety margin
        let rate_limiter = create_rate_limiter(rate_limit_type, Some(rate_limit_safety_margin));

        Self {
            base_url,
            cst,
            token: security_token,
            client_id,
            account_id,
            lightstreamer_endpoint,
            api_key,
            rate_limiter: Some(rate_limiter),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a new session with the given credentials and a rate limiter
    ///
    /// This creates a thread-safe session that can be shared across multiple threads.
    pub fn with_rate_limiter(
        cst: String,
        token: String,
        account_id: String,
        limit_type: RateLimitType,
    ) -> Self {
        Self {
            cst,
            token,
            account_id,
            base_url: String::new(),
            client_id: String::new(),
            lightstreamer_endpoint: String::new(),
            api_key: String::new(),
            rate_limiter: Some(create_rate_limiter(limit_type, Some(0.8))),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a new session with the given credentials and rate limiter configuration from Config
    pub fn from_config(cst: String, token: String, account_id: String, config: &Config) -> Self {
        Self {
            cst,
            token,
            account_id,
            base_url: String::new(),
            client_id: String::new(),
            lightstreamer_endpoint: String::new(),
            api_key: String::new(),
            rate_limiter: Some(create_rate_limiter(
                config.rate_limit_type,
                Some(config.rate_limit_safety_margin),
            )),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Waits if necessary to respect rate limits before making a request
    ///
    /// This method will always use a rate limiter - either the one configured in the session,
    /// or a default one if none is configured.
    ///
    /// This method is thread-safe and can be called from multiple threads concurrently.
    ///
    /// # Returns
    /// * `Ok(())` - If the rate limit is respected
    /// * `Err(AppError::RateLimitExceeded)` - If the rate limit has been exceeded and cannot be respected
    pub async fn respect_rate_limit(&self) -> Result<(), AppError> {
        // Mark that this session is being used in a concurrent context
        self.concurrent_mode.store(true, Ordering::SeqCst);

        // Get the rate limiter from the session or use a default one
        let limiter = match &self.rate_limiter {
            Some(limiter) => limiter.clone(),
            None => {
                // This should never happen since we always initialize with a default limiter,
                // but just in case, use the global app non-trading limiter
                debug!("No rate limiter configured in session, using default");
                app_non_trading_limiter()
            }
        };

        // Wait if necessary to respect the rate limit
        limiter.wait().await;
        Ok(())
    }

    /// Gets statistics about the current rate limit usage
    pub async fn get_rate_limit_stats(&self) -> Option<RateLimiterStats> {
        match &self.rate_limiter {
            Some(limiter) => Some(limiter.get_stats().await),
            None => None,
        }
    }
}

/// Trait for authenticating with the IG Markets API
#[async_trait::async_trait]
pub trait IgAuthenticator: Send + Sync {
    /// Logs in to the IG Markets API and returns a new session
    async fn login(&self) -> Result<IgSession, AuthError>;
    /// Refreshes an existing session with the IG Markets API
    async fn refresh(&self, session: &IgSession) -> Result<IgSession, AuthError>;
    /// Switches the active account for the current session
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `account_id` - The ID of the account to switch to
    /// * `default_account` - Whether to set this account as the default (optional)
    ///
    /// # Returns
    /// * A new session with the updated account ID
    async fn switch_account(
        &self,
        session: &IgSession,
        account_id: &str,
        default_account: Option<bool>,
    ) -> Result<IgSession, AuthError>;
}
