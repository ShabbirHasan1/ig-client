/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/

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
