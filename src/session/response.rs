/// Response structure for session-related API calls
#[derive(serde::Deserialize)]
pub struct SessionResp {
    /// Account ID associated with the session
    #[serde(alias = "accountId")]
    #[serde(alias = "currentAccountId")]
    pub account_id: String,

    /// Client ID provided by the API
    #[serde(alias = "clientId")]
    pub client_id: Option<String>,
    /// Timezone offset in hours
    #[serde(alias = "timezoneOffset")]
    pub timezone_offset: Option<i32>,
}

/// Request model for switching the active account
#[derive(serde::Serialize)]
pub struct AccountSwitchRequest {
    /// The identifier of the account being switched to
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// True if the specified account is to be set as the new default account
    #[serde(rename = "defaultAccount")]
    pub default_account: Option<bool>,
}

/// Response model for account switch operation
#[derive(serde::Deserialize, Debug)]
pub struct AccountSwitchResponse {
    /// Whether dealing is enabled for the account
    #[serde(rename = "dealingEnabled")]
    pub dealing_enabled: Option<bool>,
    /// Whether the user has active demo accounts
    #[serde(rename = "hasActiveDemoAccounts")]
    pub has_active_demo_accounts: Option<bool>,
    /// Whether the user has active live accounts
    #[serde(rename = "hasActiveLiveAccounts")]
    pub has_active_live_accounts: Option<bool>,
    /// Whether trailing stops are enabled for the account
    #[serde(rename = "trailingStopsEnabled")]
    pub trailing_stops_enabled: Option<bool>,
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
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl OAuthToken {
    /// Checks if the OAuth token is expired or will expire soon
    ///
    /// # Arguments
    /// * `margin_seconds` - Safety margin in seconds before actual expiry
    ///
    /// # Returns
    /// `true` if the token is expired or will expire within the margin, `false` otherwise
    pub fn is_expired(&self, margin_seconds: i64) -> bool {
        let expires_in_secs = self.expires_in.parse::<i64>().unwrap_or(0);
        let expiry_time = self.created_at + chrono::Duration::seconds(expires_in_secs);
        let now = chrono::Utc::now();
        let margin = chrono::Duration::seconds(margin_seconds);

        expiry_time - margin <= now
    }
}

/// Response structure for session API v3 calls
#[derive(serde::Deserialize, Debug)]
pub struct SessionV3Resp {
    /// Client ID provided by the API
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// Account ID associated with the session
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// Timezone offset in hours
    #[serde(rename = "timezoneOffset")]
    pub timezone_offset: i32,
    /// Lightstreamer endpoint for subscribing to account and price updates
    #[serde(rename = "lightstreamerEndpoint")]
    pub lightstreamer_endpoint: String,
    /// OAuth token information
    #[serde(rename = "oauthToken")]
    pub oauth_token: OAuthToken,
}
