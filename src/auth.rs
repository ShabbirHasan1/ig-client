/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/

//! Authentication module for IG Markets API
//!
//! This module provides a simplified authentication interface that handles:
//! - API v2 (CST/X-SECURITY-TOKEN) authentication
//! - API v3 (OAuth) authentication with automatic token refresh
//! - Account switching
//! - Automatic re-authentication when tokens expire

use crate::config::Config;
use crate::error::AppError;
use crate::model::responses::{SessionResponse, SessionV3Response};
use chrono::Utc;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

const USER_AGENT: &str = "ig-client/0.5.2";

/// OAuth token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    /// Access token for API requests
    pub access_token: String,
    /// Refresh token for obtaining new access tokens
    pub refresh_token: String,
    /// Token expiration time in seconds
    pub expires_in: u64,
    /// Token scope
    pub scope: String,
    /// Token type (usually "Bearer")
    pub token_type: String,
    /// Timestamp when token was created (milliseconds since epoch)
    #[serde(skip)]
    pub created_at: i64,
}

impl OAuthToken {
    /// Creates a new OAuth token with current timestamp
    pub fn new(
        access_token: String,
        refresh_token: String,
        expires_in: u64,
        scope: String,
        token_type: String,
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            expires_in,
            scope,
            token_type,
            created_at: Utc::now().timestamp(),
        }
    }

    /// Checks if the token is expired or will expire soon
    ///
    /// # Arguments
    /// * `margin_seconds` - Safety margin in seconds (default: 300 = 5 minutes)
    #[must_use]
    pub fn is_expired(&self, margin_seconds: Option<i64>) -> bool {
        let margin = margin_seconds.unwrap_or(300);
        let now = Utc::now().timestamp();
        let expires_at = self.created_at + self.expires_in as i64;
        now >= (expires_at - margin)
    }
}

/// Session information for authenticated requests
#[derive(Debug, Clone)]
pub struct Session {
    /// Account ID
    pub account_id: String,
    /// Client ID (for OAuth)
    pub client_id: Option<String>,
    /// Lightstreamer endpoint
    pub lightstreamer_endpoint: Option<String>,
    /// CST token (API v2)
    pub cst: Option<String>,
    /// X-SECURITY-TOKEN (API v2)
    pub x_security_token: Option<String>,
    /// OAuth token (API v3)
    pub oauth_token: Option<OAuthToken>,
    /// API version used
    pub api_version: u8,
}

impl Session {
    /// Checks if this session uses OAuth authentication
    #[must_use]
    pub fn is_oauth(&self) -> bool {
        self.oauth_token.is_some()
    }

    /// Checks if OAuth token needs refresh
    ///
    /// # Arguments
    /// * `margin_seconds` - Safety margin in seconds (default: 300 = 5 minutes)
    #[must_use]
    pub fn needs_token_refresh(&self, margin_seconds: Option<i64>) -> bool {
        if let Some(oauth) = &self.oauth_token {
            oauth.is_expired(margin_seconds)
        } else {
            false
        }
    }
}

/// Authentication manager for IG Markets API
///
/// Handles all authentication operations including:
/// - Login with API v2 or v3
/// - Automatic OAuth token refresh
/// - Account switching
/// - Session management
pub struct Auth {
    config: Arc<Config>,
    client: Client,
    session: Arc<RwLock<Option<Session>>>,
}

impl Auth {
    /// Creates a new Auth instance
    ///
    /// # Arguments
    /// * `config` - Configuration containing credentials and API settings
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            session: Arc::new(RwLock::new(None)),
        }
    }

    /// Gets the current session, ensuring tokens are valid
    ///
    /// This method automatically refreshes expired OAuth tokens or re-authenticates if needed.
    ///
    /// # Returns
    /// * `Ok(Session)` - Valid session with fresh tokens
    /// * `Err(AppError)` - If authentication fails
    pub async fn get_session(&self) -> Result<Session, AppError> {
        let session = self.session.read().await;

        if let Some(sess) = session.as_ref() {
            // Check if OAuth token needs refresh
            if sess.needs_token_refresh(Some(300)) {
                drop(session); // Release read lock
                info!("OAuth token needs refresh");
                return self.refresh_token().await;
            }
            return Ok(sess.clone());
        }

        drop(session);

        // No session exists, need to login
        info!("No active session, logging in");
        self.login().await
    }

    /// Performs initial login to IG Markets API
    ///
    /// Automatically detects API version from config and uses appropriate authentication method.
    ///
    /// # Returns
    /// * `Ok(Session)` - Authenticated session
    /// * `Err(AppError)` - If login fails
    pub async fn login(&self) -> Result<Session, AppError> {
        let api_version = self.config.api_version.unwrap_or(2);

        info!("Logging in with API v{}", api_version);

        let session = if api_version == 3 {
            self.login_oauth().await?
        } else {
            self.login_v2().await?
        };

        // Store session
        let mut sess = self.session.write().await;
        *sess = Some(session.clone());

        info!("✓ Login successful, account: {}", session.account_id);
        Ok(session)
    }

    /// Performs login using API v2 (CST/X-SECURITY-TOKEN)
    async fn login_v2(&self) -> Result<Session, AppError> {
        let url = format!("{}/session", self.config.rest_api.base_url);

        let body = serde_json::json!({
            "identifier": self.config.credentials.username,
            "password": self.config.credentials.password,
        });

        debug!("Sending v2 login request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("X-IG-API-KEY", &self.config.credentials.api_key)
            .header("Content-Type", "application/json")
            .header("Version", "2")
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let cst = response
            .headers()
            .get("CST")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let x_security_token = response
            .headers()
            .get("X-SECURITY-TOKEN")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if status != StatusCode::OK {
            let body = response.text().await.unwrap_or_default();
            error!("Login failed with status {}: {}", status, body);
            return Err(AppError::Unauthorized);
        }

        let json: SessionResponse = response.json().await?;

        Ok(Session {
            account_id: json.account_id,
            client_id: json.client_id,
            lightstreamer_endpoint: json.lightstreamer_endpoint,
            cst,
            x_security_token,
            oauth_token: None,
            api_version: 2,
        })
    }

    /// Performs login using API v3 (OAuth)
    async fn login_oauth(&self) -> Result<Session, AppError> {
        let url = format!("{}/session", self.config.rest_api.base_url);

        let body = serde_json::json!({
            "identifier": self.config.credentials.username,
            "password": self.config.credentials.password,
        });

        debug!("Sending OAuth login request to: {}", url);

        let response = self
            .client
            .post(&url)
            .header("X-IG-API-KEY", &self.config.credentials.api_key)
            .header("Content-Type", "application/json")
            .header("Version", "3")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if status != StatusCode::OK {
            let body = response.text().await.unwrap_or_default();
            error!("OAuth login failed with status {}: {}", status, body);
            return Err(AppError::Unauthorized);
        }

        let json: SessionV3Response = response.json().await?;

        debug!(
            "OAuth token expires in {} seconds",
            json.oauth_token.expires_in
        );

        // Convert the expires_in string to u64
        let expires_in = json.oauth_token.expires_in.parse::<u64>().unwrap_or(600);

        let oauth_token = OAuthToken::new(
            json.oauth_token.access_token,
            json.oauth_token.refresh_token,
            expires_in,
            json.oauth_token.scope,
            json.oauth_token.token_type,
        );

        Ok(Session {
            account_id: json.account_id,
            client_id: Some(json.client_id),
            lightstreamer_endpoint: Some(json.lightstreamer_endpoint),
            cst: None,
            x_security_token: None,
            oauth_token: Some(oauth_token),
            api_version: 3,
        })
    }

    /// Refreshes an expired OAuth token
    ///
    /// If refresh fails (e.g., refresh token expired), performs full re-authentication.
    ///
    /// # Returns
    /// * `Ok(Session)` - New session with refreshed tokens
    /// * `Err(AppError)` - If refresh and re-authentication both fail
    pub async fn refresh_token(&self) -> Result<Session, AppError> {
        let current_session = {
            let session = self.session.read().await;
            session.clone()
        };

        let Some(sess) = current_session else {
            warn!("No session to refresh, performing login");
            return self.login().await;
        };

        let Some(oauth) = &sess.oauth_token else {
            warn!("Not an OAuth session, cannot refresh");
            return Ok(sess);
        };

        info!("Refreshing OAuth token");

        let url = format!("{}/session/refresh-token", self.config.rest_api.base_url);

        let body = serde_json::json!({
            "refresh_token": oauth.refresh_token,
        });

        let response = self
            .client
            .post(&url)
            .header("X-IG-API-KEY", &self.config.credentials.api_key)
            .header("Content-Type", "application/json")
            .header("Version", "1")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if status != StatusCode::OK {
            let body = response.text().await.unwrap_or_default();
            warn!("Token refresh failed with status {}: {}", status, body);
            warn!("Refresh token may be expired, attempting full re-authentication");

            // Refresh token expired, need to login again
            return self.login().await;
        }

        // Parse the refresh token response (only contains OAuth token, not full session)
        #[derive(Deserialize)]
        struct RefreshTokenResponse {
            access_token: String,
            refresh_token: String,
            expires_in: String,
            scope: String,
            token_type: String,
        }

        let token_response: RefreshTokenResponse = response.json().await?;

        let expires_in = token_response.expires_in.parse::<u64>().unwrap_or(600);

        let oauth_token = OAuthToken::new(
            token_response.access_token,
            token_response.refresh_token,
            expires_in,
            token_response.scope,
            token_response.token_type,
        );

        // Update the OAuth token in the existing session, keeping other fields
        let new_session = Session {
            account_id: sess.account_id.clone(),
            client_id: sess.client_id.clone(),
            lightstreamer_endpoint: sess.lightstreamer_endpoint.clone(),
            cst: None,
            x_security_token: None,
            oauth_token: Some(oauth_token),
            api_version: 3,
        };

        // Update stored session
        let mut session = self.session.write().await;
        *session = Some(new_session.clone());

        info!("✓ Token refreshed successfully");
        Ok(new_session)
    }

    /// Switches to a different trading account
    ///
    /// # Arguments
    /// * `account_id` - The account ID to switch to
    /// * `default_account` - Whether to set as default account
    ///
    /// # Returns
    /// * `Ok(Session)` - New session for the switched account
    /// * `Err(AppError)` - If account switch fails
    pub async fn switch_account(
        &self,
        account_id: &str,
        default_account: Option<bool>,
    ) -> Result<Session, AppError> {
        let current_session = self.get_session().await?;

        if current_session.account_id == account_id {
            debug!("Already on account {}", account_id);
            return Ok(current_session);
        }

        info!("Switching to account: {}", account_id);

        let url = format!("{}/session", self.config.rest_api.base_url);

        let mut body = serde_json::json!({
            "accountId": account_id,
        });

        if let Some(default) = default_account {
            body["defaultAccount"] = serde_json::json!(default);
        }

        let mut request = self
            .client
            .put(&url)
            .header("X-IG-API-KEY", &self.config.credentials.api_key)
            .header("Content-Type", "application/json")
            .header("Version", "1")
            .json(&body);

        // Add authentication headers
        if let Some(oauth) = &current_session.oauth_token {
            request = request.header("Authorization", format!("Bearer {}", oauth.access_token));
        } else {
            if let Some(cst) = &current_session.cst {
                request = request.header("CST", cst);
            }
            if let Some(token) = &current_session.x_security_token {
                request = request.header("X-SECURITY-TOKEN", token);
            }
        }

        let response = request.send().await?;

        let status = response.status();

        if status != StatusCode::OK {
            let body = response.text().await.unwrap_or_default();
            error!("Account switch failed with status {}: {}", status, body);
            return Err(AppError::Unexpected(status));
        }

        // After switching, update the session
        let mut new_session = current_session.clone();
        new_session.account_id = account_id.to_string();

        let mut session = self.session.write().await;
        *session = Some(new_session.clone());

        info!("✓ Switched to account: {}", account_id);
        Ok(new_session)
    }

    /// Logs out and clears the current session
    pub async fn logout(&self) -> Result<(), AppError> {
        info!("Logging out");

        let mut session = self.session.write().await;
        *session = None;

        info!("✓ Logged out successfully");
        Ok(())
    }
}
