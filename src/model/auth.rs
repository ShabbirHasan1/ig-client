/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::warn;
use crate::application::auth::Session;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SessionResponse {
    V3(V3Response),
    V2(V2Response),
}

impl SessionResponse {
    pub fn is_v3(&self) -> bool {
        matches!(self, SessionResponse::V3(_))
    }
    pub fn is_v2(&self) -> bool {
        matches!(self, SessionResponse::V2(_))
    }
    pub fn get_session(&self) -> Session {
        match self { 
            SessionResponse::V3(v) => {
                Session {
                    account_id: v.account_id.clone(),
                    client_id: v.client_id.clone(),
                    lightstreamer_endpoint: v.lightstreamer_endpoint.clone(),
                    cst: None,
                    x_security_token: None,
                    oauth_token: Some(v.oauth_token.clone()),
                    api_version: 3,
                    expires_at: v.oauth_token.expire_at(1)
                }
            },
            SessionResponse::V2(v) => {
                let (cst, x_security_token) = match v.security_headers.as_ref() {
                    Some(headers) => (Some(headers.cst.clone()), Some(headers.x_security_token.clone())),
                    None => (None, None)
                };
                let expires_at = (Utc::now().timestamp() + (3600 * 6)) as u64; // 6 hours from now
                Session {
                    account_id: v.current_account_id.clone(),
                    client_id: v.client_id.clone(),
                    lightstreamer_endpoint: v.lightstreamer_endpoint.clone(),
                    cst,
                    x_security_token,
                    oauth_token: None,
                    api_version: 2,
                    expires_at
                }
            },
        }
    }
    pub fn get_session_v2(&mut self, headers: &SecurityHeaders) -> Session {
        match self { 
            SessionResponse::V3(_) => {
                warn!("Returing V3 session from V2 headers - this may be unexpected");
                self.get_session()
            },
            SessionResponse::V2(v) => {
                v.set_security_headers(headers);
                v.expires_in = Some(21600); // 6 hours
                self.get_session()
            }
        }
    }
    pub fn is_expired(&self, margin_seconds: u64) -> bool {
        match self {
            SessionResponse::V3(v) => {
                v.oauth_token.is_expired(margin_seconds)
            },
            SessionResponse::V2(v) => {
                v.is_expired(margin_seconds)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V3Response {
    pub client_id: String,
    pub account_id: String,
    pub timezone_offset: i32,
    pub lightstreamer_endpoint: String,
    pub oauth_token: OAuthToken,
}

/// OAuth token information returned by API v3
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OAuthToken {
    /// OAuth access token
    pub access_token: String,
    /// OAuth refresh token
    pub refresh_token: String,
    /// Token scope
    pub scope: String,
    /// Token type (typically "Bearer")
    pub token_type: String,
    /// Token expiry time in seconds
    pub expires_in: String,
    /// Timestamp when this token was created (for expiry calculation)
    #[serde(skip, default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<Utc>,
}

impl OAuthToken {
    /// Checks if the OAuth token is expired or will expire soon
    ///
    /// # Arguments
    /// * `margin_seconds` - Safety margin in seconds before actual expiry
    ///
    /// # Returns
    /// `true` if the token is expired or will expire within the margin, `false` otherwise
    pub fn is_expired(&self, margin_seconds: u64) -> bool {
        let expires_in_secs = self.expires_in.parse::<i64>().unwrap_or(0);
        let expiry_time = self.created_at + chrono::Duration::seconds(expires_in_secs);
        let now = Utc::now();
        let margin = chrono::Duration::seconds(margin_seconds as i64);

        expiry_time - margin <= now
    }

    /// Returns the Unix timestamp when the token expires (considering the margin)
    ///
    /// # Arguments
    /// * `margin_seconds` - Safety margin in seconds before actual expiry
    ///
    /// # Returns
    /// Unix timestamp (seconds since epoch) when the token should be considered expired
    pub fn expire_at(&self, margin_seconds: i64) -> u64 {
        let expires_in_secs = self.expires_in.parse::<i64>().unwrap_or(0);
        let expiry_time = self.created_at + chrono::Duration::seconds(expires_in_secs);
        let margin = chrono::Duration::seconds(margin_seconds);

        // Subtract margin to get the "effective" expiry time
        let effective_expiry = expiry_time - margin;

        effective_expiry.timestamp() as u64
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct V2Response {
    pub account_type: String,
    pub account_info: AccountInfo,
    pub currency_iso_code: String,
    pub currency_symbol: String,
    pub current_account_id: String,
    pub lightstreamer_endpoint: String,
    pub accounts: Vec<Account>,
    pub client_id: String,
    pub timezone_offset: i32,
    pub has_active_demo_accounts: bool,
    pub has_active_live_accounts: bool,
    pub trailing_stops_enabled: bool,
    pub rerouting_environment: Option<String>,
    pub dealing_enabled: bool,
    #[serde(skip)]
    pub security_headers: Option<SecurityHeaders>,
    /// Token expiry time in seconds
    #[serde(skip)]
    pub expires_in: Option<u64>,
    /// Timestamp when this token was created (for expiry calculation)
    #[serde(skip, default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<Utc>,
}

impl V2Response {
    pub fn set_security_headers(&mut self, headers: &SecurityHeaders) {
        self.security_headers = Some(headers.clone());
    }
    
    pub fn is_expired(&self, margin_seconds: u64) -> bool {
        if let Some(expires_in) = self.expires_in {
            let expiry_time = self.created_at + chrono::Duration::seconds(expires_in as i64);
            let now = Utc::now();
            let margin = chrono::Duration::seconds(margin_seconds as i64);

            expiry_time - margin <= now
        } else {
            panic!("expires_in not set in V2Response");
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityHeaders {
    pub cst: String,
    pub x_security_token: String,
    pub x_ig_api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub balance: f64,
    pub deposit: f64,
    pub profit_loss: f64,
    pub available: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub account_id: String,
    pub account_name: String,
    pub preferred: bool,
    pub account_type: String,
}