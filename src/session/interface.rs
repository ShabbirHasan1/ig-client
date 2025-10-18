use crate::config::Config;
use crate::error::{AppError, AuthError};
use crate::session::response::OAuthToken;
use crate::utils::rate_limiter::{
    RateLimitType, RateLimiter, RateLimiterStats, app_non_trading_limiter, create_rate_limiter,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::debug;

/// Timer for managing IG API token expiration and refresh cycles
///
/// According to IG API documentation, tokens are initially valid for 6 hours
/// but get extended up to a maximum of 72 hours while they are in use.
#[derive(Debug, Clone)]
pub struct TokenTimer {
    /// The current expiry time of the token (initially 6 hours from creation)
    pub expiry: DateTime<Utc>,
    /// The timestamp when the token was last refreshed
    pub last_refreshed: DateTime<Utc>,
    /// The maximum age the token can reach (72 hours from initial creation)
    pub max_age: DateTime<Utc>,
}

impl Default for TokenTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenTimer {
    /// Creates a new TokenTimer with initial 6-hour expiry and 72-hour maximum age
    ///
    /// # Returns
    /// A new TokenTimer instance with expiry set to 6 hours from now and max_age set to 72 hours from now
    pub fn new() -> Self {
        let expiry = Utc::now() + chrono::Duration::hours(6);
        let max_age = Utc::now() + chrono::Duration::hours(72);
        Self {
            expiry,
            last_refreshed: Utc::now(),
            max_age,
        }
    }

    /// Checks if the token is expired based on current time
    ///
    /// # Returns
    /// `true` if either the token expiry time or maximum age has been reached, `false` otherwise
    pub fn is_expired(&self) -> bool {
        self.expiry <= Utc::now() || self.max_age <= Utc::now()
    }

    /// Checks if the token is expired or will expire within the given margin
    ///
    /// # Arguments
    /// * `margin` - The time margin to check before actual expiry
    ///
    /// # Returns
    /// `true` if the token will expire within the margin or has already expired, `false` otherwise
    pub fn is_expired_w_margin(&self, margin: chrono::Duration) -> bool {
        self.expiry - margin <= Utc::now() || self.max_age - margin <= Utc::now()
    }

    /// Refreshes the token timer, extending the expiry time by 6 hours from now
    ///
    /// This should be called after each successful API request to extend token validity.
    /// The expiry time is reset to 6 hours from the current time, but cannot exceed max_age.
    pub fn refresh(&mut self) {
        self.expiry = Utc::now() + chrono::Duration::hours(6);
        self.last_refreshed = Utc::now();
    }
}

/// Session information for IG Markets API authentication
///
/// Supports both API v2 (CST/X-SECURITY-TOKEN) and v3 (OAuth) authentication.
#[derive(Debug, Clone)]
pub struct IgSession {
    /// Client Session Token (CST) used for authentication (API v2)
    pub cst: String,
    /// Security token used for authentication (API v2)
    pub token: String,
    /// OAuth token information (API v3)
    pub oauth_token: Option<OAuthToken>,
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
    /// Timer for managing token expiration and automatic refresh cycles
    pub token_timer: Arc<Mutex<TokenTimer>>,
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
            oauth_token: None,
            client_id: String::new(),
            account_id,
            lightstreamer_endpoint: String::new(),
            api_key: String::new(),
            rate_limiter: Some(create_rate_limiter(
                RateLimitType::NonTradingAccount,
                Some(0.8),
            )),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
            token_timer: Arc::new(Mutex::new(TokenTimer::new())),
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
            oauth_token: None,
            client_id,
            account_id,
            lightstreamer_endpoint,
            api_key,
            rate_limiter: Some(rate_limiter),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
            token_timer: Arc::new(Mutex::new(TokenTimer::new())),
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
            oauth_token: None,
            account_id,
            base_url: String::new(),
            client_id: String::new(),
            lightstreamer_endpoint: String::new(),
            api_key: String::new(),
            rate_limiter: Some(create_rate_limiter(limit_type, Some(0.8))),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
            token_timer: Arc::new(Mutex::new(TokenTimer::new())),
        }
    }

    /// Creates a new session with the given credentials and rate limiter configuration from Config
    pub fn from_config(cst: String, token: String, account_id: String, config: &Config) -> Self {
        Self {
            cst,
            token,
            oauth_token: None,
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
            token_timer: Arc::new(Mutex::new(TokenTimer::new())),
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

    /// Refreshes the token timer to extend token validity
    /// This should be called after each successful API request
    pub fn refresh_token_timer(&self) {
        if let Ok(mut timer) = self.token_timer.lock() {
            timer.refresh();
        }
    }

    /// Checks if this session is using OAuth (API v3) authentication
    ///
    /// # Returns
    /// `true` if the session has OAuth tokens, `false` otherwise
    pub fn is_oauth(&self) -> bool {
        self.oauth_token.is_some()
    }

    /// Checks if this session is using CST/X-SECURITY-TOKEN (API v2) authentication
    ///
    /// # Returns
    /// `true` if the session uses CST tokens, `false` otherwise
    pub fn is_cst_auth(&self) -> bool {
        !self.cst.is_empty() && !self.token.is_empty() && self.oauth_token.is_none()
    }

    /// Creates a new session with OAuth authentication (API v3)
    ///
    /// # Arguments
    /// * `oauth_token` - The OAuth token information
    /// * `account_id` - Account ID associated with the session
    /// * `client_id` - Client ID provided by the API
    /// * `lightstreamer_endpoint` - Lightstreamer endpoint for real-time data
    /// * `config` - Configuration for rate limiting
    ///
    /// # Returns
    /// A new IgSession configured for OAuth authentication
    pub fn from_oauth(
        oauth_token: OAuthToken,
        account_id: String,
        client_id: String,
        lightstreamer_endpoint: String,
        config: &Config,
    ) -> Self {
        Self {
            cst: String::new(),
            token: String::new(),
            oauth_token: Some(oauth_token),
            account_id,
            base_url: config.rest_api.base_url.clone(),
            client_id,
            lightstreamer_endpoint,
            api_key: config.credentials.api_key.clone(),
            rate_limiter: Some(create_rate_limiter(
                config.rate_limit_type,
                Some(config.rate_limit_safety_margin),
            )),
            concurrent_mode: Arc::new(AtomicBool::new(false)),
            token_timer: Arc::new(Mutex::new(TokenTimer::new())),
        }
    }
}

/// Trait for authenticating with the IG Markets API
#[async_trait::async_trait]
pub trait IgAuthenticator: Send + Sync {
    /// Logs in to the IG Markets API and returns a new session
    ///
    /// Automatically selects API v2 or v3 based on configuration.
    /// Defaults to v3 (OAuth) if not specified.
    async fn login(&self) -> Result<IgSession, AuthError>;

    /// Logs in using API v2 (CST/X-SECURITY-TOKEN authentication)
    ///
    /// # Returns
    /// * `Ok(IgSession)` - A new session with CST and X-SECURITY-TOKEN
    /// * `Err(AuthError)` - If authentication fails
    async fn login_v2(&self) -> Result<IgSession, AuthError>;

    /// Logs in using API v3 (OAuth authentication)
    ///
    /// # Returns
    /// * `Ok(IgSession)` - A new session with OAuth tokens
    /// * `Err(AuthError)` - If authentication fails
    async fn login_v3(&self) -> Result<IgSession, AuthError>;

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

    /// Attempts to login and switch to the specified account, optionally setting it as the default account.
    ///
    /// # Arguments
    ///
    /// * `account_id` - A string slice that holds the ID of the account to which the session should switch.
    /// * `default_account` - An optional boolean parameter. If `Some(true)`, the given account will be marked
    ///   as the default account for subsequent operations. If `None` or `Some(false)`, the account will not
    ///   be set as default.
    ///
    /// # Returns
    ///
    /// This function returns a `Result`:
    /// * `Ok(IgSession)` - On success, contains an updated `IgSession` object representing the active session
    ///   state after the switch.
    /// * `Err(AuthError)` - If the operation fails, returns an `AuthError` containing details about the issue.
    ///
    /// # Errors
    ///
    /// This function can return `AuthError` in the following scenarios:
    /// * If the provided `account_id` is invalid or does not exist.
    /// * If there is a network issue during the login/switch process.
    /// * If there are authentication or session-related failures.
    ///
    /// # Notes
    ///
    /// Ensure that the `account_id` is valid and accessible under the authenticated user's account scope.
    /// Switching accounts may invalidate the previous session if the platform enforces single-session
    /// restrictions.
    async fn login_and_switch_account(
        &self,
        account_id: &str,
        default_account: Option<bool>,
    ) -> Result<IgSession, AuthError>;

    /// Attempts to relogin (if needed) and switch to the specified account.
    /// This method uses relogin() instead of login() to avoid unnecessary authentication
    /// when tokens are still valid.
    ///
    /// # Arguments
    /// * `session` - The current session to check for token validity
    /// * `account_id` - The ID of the account to switch to
    /// * `default_account` - Whether to set this account as the default (optional)
    ///
    /// # Returns
    /// * `Ok(IgSession)` - On success, contains an updated session for the target account
    /// * `Err(AuthError)` - If the operation fails
    async fn relogin_and_switch_account(
        &self,
        session: &IgSession,
        account_id: &str,
        default_account: Option<bool>,
    ) -> Result<IgSession, AuthError>;

    /// Re-authenticates only if the current session tokens are expired or close to expiring.
    /// This method checks the token expiration with a safety margin and only performs login
    /// if necessary, making it more efficient than always calling login().
    ///
    /// # Arguments
    /// * `session` - The current session to check for token validity
    ///
    /// # Returns
    /// * `Ok(IgSession)` - Either the existing session (if tokens are still valid) or a new session (if re-login was needed)
    /// * `Err(AuthError)` - If re-authentication fails
    async fn relogin(&self, session: &IgSession) -> Result<IgSession, AuthError>;
}
