use crate::constants::{DAYS_TO_BACK_LOOK, DEFAULT_PAGE_SIZE, DEFAULT_SLEEP_TIME};
use crate::impl_json_display;
use crate::storage::config::DatabaseConfig;
use crate::utils::rate_limiter::RateLimitType;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fmt::Debug;
use std::str::FromStr;
use tracing::error;
use tracing::log::debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Authentication credentials for the IG Markets API
pub struct Credentials {
    /// Username for the IG Markets account
    pub username: String,
    /// Password for the IG Markets account
    pub password: String,
    /// Account ID for the IG Markets account
    pub account_id: String,
    /// API key for the IG Markets API
    pub api_key: String,
    /// Client token for the IG Markets API
    pub client_token: Option<String>,
    /// Account token for the IG Markets API
    pub account_token: Option<String>,
}

impl_json_display!(Credentials);

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Main configuration for the IG Markets API client
pub struct Config {
    /// Authentication credentials
    pub credentials: Credentials,
    /// REST API configuration
    pub rest_api: RestApiConfig,
    /// WebSocket API configuration
    pub websocket: WebSocketConfig,
    /// Database configuration for data persistence
    pub database: DatabaseConfig,
    /// Number of hours between transaction fetching operations
    pub sleep_hours: u64,
    /// Number of items to retrieve per page in API requests
    pub page_size: u32,
    /// Number of days to look back when fetching historical data
    pub days_to_look_back: i64,
    /// Rate limit type to use for API requests
    pub rate_limit_type: RateLimitType,
    /// Safety margin for rate limiting (0.0-1.0)
    pub rate_limit_safety_margin: f64,
    /// API version to use for authentication (2 or 3). If None, auto-detect based on available tokens
    pub api_version: Option<u8>,
}

impl_json_display!(Config);

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Configuration for the REST API
pub struct RestApiConfig {
    /// Base URL for the IG Markets REST API
    pub base_url: String,
    /// Timeout in seconds for REST API requests
    pub timeout: u64,
}

impl_json_display!(RestApiConfig);

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Configuration for the WebSocket API
pub struct WebSocketConfig {
    /// URL for the IG Markets WebSocket API
    pub url: String,
    /// Reconnect interval in seconds for WebSocket connections
    pub reconnect_interval: u64,
}

impl_json_display!(WebSocketConfig);

/// Gets an environment variable or returns a default value if not found or cannot be parsed
///
/// # Arguments
///
/// * `env_var` - The name of the environment variable
/// * `default` - The default value to use if the environment variable is not found or cannot be parsed
///
/// # Returns
///
/// The parsed value of the environment variable or the default value
pub fn get_env_or_default<T: FromStr>(env_var: &str, default: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    match env::var(env_var) {
        Ok(val) => val.parse::<T>().unwrap_or_else(|_| {
            error!("Failed to parse {}: {}, using default", env_var, val);
            default
        }),
        Err(_) => default,
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Creates a new configuration instance from environment variables
    ///
    /// Loads configuration from environment variables or .env file.
    /// Uses default values if environment variables are not found.
    ///
    /// # Returns
    ///
    /// A new `Config` instance
    pub fn new() -> Self {
        Self::with_rate_limit_type(RateLimitType::OnePerSecond, 0.5)
    }

    /// Creates a new configuration instance with a specific rate limit type
    ///
    /// # Arguments
    ///
    /// * `rate_limit_type` - The type of rate limit to enforce
    /// * `safety_margin` - A value between 0.0 and 1.0 representing the percentage of the actual limit to use
    ///
    /// # Returns
    ///
    /// A new `Config` instance
    pub fn with_rate_limit_type(rate_limit_type: RateLimitType, safety_margin: f64) -> Self {
        // Explicitly load the .env file
        match dotenv() {
            Ok(_) => debug!("Successfully loaded .env file"),
            Err(e) => debug!("Failed to load .env file: {e}"),
        }

        // Check if environment variables are configured
        let username = get_env_or_default("IG_USERNAME", String::from("default_username"));
        let password = get_env_or_default("IG_PASSWORD", String::from("default_password"));
        let api_key = get_env_or_default("IG_API_KEY", String::from("default_api_key"));

        let sleep_hours = get_env_or_default("TX_LOOP_INTERVAL_HOURS", DEFAULT_SLEEP_TIME);
        let page_size = get_env_or_default("TX_PAGE_SIZE", DEFAULT_PAGE_SIZE);
        let days_to_look_back = get_env_or_default("TX_DAYS_LOOKBACK", DAYS_TO_BACK_LOOK);

        // Check if we are using default values
        if username == "default_username" {
            error!("IG_USERNAME not found in environment variables or .env file");
        }
        if password == "default_password" {
            error!("IG_PASSWORD not found in environment variables or .env file");
        }
        if api_key == "default_api_key" {
            error!("IG_API_KEY not found in environment variables or .env file");
        }

        // Print information about loaded environment variables
        debug!("Environment variables loaded:");
        debug!(
            "  IG_USERNAME: {}",
            if username == "default_username" {
                "Not set"
            } else {
                "Set"
            }
        );
        debug!(
            "  IG_PASSWORD: {}",
            if password == "default_password" {
                "Not set"
            } else {
                "Set"
            }
        );
        debug!(
            "  IG_API_KEY: {}",
            if api_key == "default_api_key" {
                "Not set"
            } else {
                "Set"
            }
        );

        // Ensure safety margin is within valid range
        let safety_margin = safety_margin.clamp(0.1, 1.0);

        Config {
            credentials: Credentials {
                username,
                password,
                account_id: get_env_or_default("IG_ACCOUNT_ID", String::from("default_account_id")),
                api_key,
                client_token: None,
                account_token: None,
            },
            rest_api: RestApiConfig {
                base_url: get_env_or_default(
                    "IG_REST_BASE_URL",
                    String::from("https://demo-api.ig.com/gateway/deal"),
                ),
                timeout: get_env_or_default("IG_REST_TIMEOUT", 30),
            },
            websocket: WebSocketConfig {
                url: get_env_or_default(
                    "IG_WS_URL",
                    String::from("wss://demo-apd.marketdatasystems.com"),
                ),
                reconnect_interval: get_env_or_default("IG_WS_RECONNECT_INTERVAL", 5),
            },
            database: DatabaseConfig {
                url: get_env_or_default(
                    "DATABASE_URL",
                    String::from("postgres://postgres:postgres@localhost/ig"),
                ),
                max_connections: get_env_or_default("DATABASE_MAX_CONNECTIONS", 5),
            },
            sleep_hours,
            page_size,
            days_to_look_back,
            rate_limit_type,
            rate_limit_safety_margin: safety_margin,
            api_version: env::var("IG_API_VERSION")
                .ok()
                .and_then(|v| v.parse::<u8>().ok())
                .filter(|&v| v == 2 || v == 3),
        }
    }

    /// Creates a PostgreSQL connection pool using the database configuration
    ///
    /// # Returns
    ///
    /// A Result containing either a PostgreSQL connection pool or an error
    pub async fn pg_pool(&self) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(self.database.max_connections)
            .connect(&self.database.url)
            .await
    }
}

#[cfg(test)]
mod tests_display {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    #[test]
    fn test_credentials_display() {
        let credentials = Credentials {
            username: "user123".to_string(),
            password: "pass123".to_string(),
            account_id: "acc456".to_string(),
            api_key: "key789".to_string(),
            client_token: Some("ctoken".to_string()),
            account_token: None,
        };

        let display_output = credentials.to_string();
        let expected_json = json!({
            "username": "user123",
            "password": "pass123",
            "account_id": "acc456",
            "api_key": "key789",
            "client_token": "ctoken",
            "account_token": null
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_rest_api_config_display() {
        let rest_api_config = RestApiConfig {
            base_url: "https://api.example.com".to_string(),
            timeout: 30,
        };

        let display_output = rest_api_config.to_string();
        let expected_json = json!({
            "base_url": "https://api.example.com",
            "timeout": 30
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_websocket_config_display() {
        let websocket_config = WebSocketConfig {
            url: "wss://ws.example.com".to_string(),
            reconnect_interval: 5,
        };

        let display_output = websocket_config.to_string();
        let expected_json = json!({
            "url": "wss://ws.example.com",
            "reconnect_interval": 5
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }

    #[test]
    fn test_config_display() {
        let config = Config {
            credentials: Credentials {
                username: "user123".to_string(),
                password: "pass123".to_string(),
                account_id: "acc456".to_string(),
                api_key: "key789".to_string(),
                client_token: Some("ctoken".to_string()),
                account_token: None,
            },
            rest_api: RestApiConfig {
                base_url: "https://api.example.com".to_string(),
                timeout: 30,
            },
            websocket: WebSocketConfig {
                url: "wss://ws.example.com".to_string(),
                reconnect_interval: 5,
            },
            database: DatabaseConfig {
                url: "postgres://user:pass@localhost/ig_db".to_string(),
                max_connections: 5,
            },
            sleep_hours: 0,
            page_size: 0,
            days_to_look_back: 0,
            rate_limit_type: RateLimitType::NonTradingAccount,
            rate_limit_safety_margin: 0.8,
            api_version: None,
        };

        let display_output = config.to_string();
        let expected_json = json!({
            "credentials": {
                "username": "user123",
                "password": "pass123",
                "account_id": "acc456",
                "api_key": "key789",
                "client_token": "ctoken",
                "account_token": null
            },
            "rest_api": {
                "base_url": "https://api.example.com",
                "timeout": 30
            },
            "websocket": {
                "url": "wss://ws.example.com",
                "reconnect_interval": 5
            },
            "database": {
                "url": "postgres://user:pass@localhost/ig_db",
                "max_connections": 5
            },
            "sleep_hours": 0,
            "page_size": 0,
            "days_to_look_back": 0,
            "rate_limit_type": "NonTradingAccount",
            "rate_limit_safety_margin": 0.8,
            "api_version": null
        });

        assert_json_eq!(
            serde_json::from_str::<serde_json::Value>(&display_output).unwrap(),
            expected_json
        );
    }
}
