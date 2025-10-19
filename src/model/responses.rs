/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use crate::model::auth::OAuthToken;

/// Response structure for session API v3 calls
#[derive(serde::Deserialize, Debug)]
pub struct SessionV3Response {
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

/// Response structure for session-related API calls
#[derive(serde::Deserialize)]
pub struct SessionResponse {
    /// Account ID associated with the session
    #[serde(alias = "accountId")]
    #[serde(alias = "currentAccountId")]
    pub account_id: String,

    /// Client ID provided by the API
    #[serde(alias = "clientId", default)]
    pub client_id: Option<String>,

    /// Timezone offset in hours
    #[serde(alias = "timezoneOffset", default)]
    pub timezone_offset: Option<i32>,

    /// Lightstreamer endpoint for real-time data
    #[serde(alias = "lightstreamerEndpoint", default)]
    pub lightstreamer_endpoint: Option<String>,
}
