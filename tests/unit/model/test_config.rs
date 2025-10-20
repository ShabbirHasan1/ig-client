use ig_client::application::config::{
    Config, Credentials, RateLimiterConfig, RestApiConfig, WebSocketConfig,
};
use ig_client::storage::config::DatabaseConfig;

#[test]
fn test_credentials_clone() {
    let creds = Credentials {
        username: "test_user".to_string(),
        password: "test_pass".to_string(),
        account_id: "ACC123".to_string(),
        api_key: "key123".to_string(),
        client_token: Some("client_token".to_string()),
        account_token: Some("account_token".to_string()),
    };

    let cloned = creds.clone();
    assert_eq!(creds.username, cloned.username);
    assert_eq!(creds.password, cloned.password);
    assert_eq!(creds.account_id, cloned.account_id);
    assert_eq!(creds.api_key, cloned.api_key);
}

#[test]
fn test_credentials_serialization() {
    let creds = Credentials {
        username: "test_user".to_string(),
        password: "test_pass".to_string(),
        account_id: "ACC123".to_string(),
        api_key: "key123".to_string(),
        client_token: None,
        account_token: None,
    };

    let json = serde_json::to_string(&creds).unwrap();
    let deserialized: Credentials = serde_json::from_str(&json).unwrap();

    assert_eq!(creds.username, deserialized.username);
    assert_eq!(creds.api_key, deserialized.api_key);
}

#[test]
fn test_rest_api_config_clone() {
    let config = RestApiConfig {
        base_url: "https://api.example.com".to_string(),
        timeout: 30,
    };

    let cloned = config.clone();
    assert_eq!(config.base_url, cloned.base_url);
    assert_eq!(config.timeout, cloned.timeout);
}

#[test]
fn test_websocket_config_clone() {
    let config = WebSocketConfig {
        url: "wss://ws.example.com".to_string(),
        reconnect_interval: 5,
    };

    let cloned = config.clone();
    assert_eq!(config.url, cloned.url);
    assert_eq!(config.reconnect_interval, cloned.reconnect_interval);
}

#[test]
fn test_rate_limiter_config_clone() {
    let config = RateLimiterConfig {
        max_requests: 10,
        period_seconds: 60,
        burst_size: 5,
    };

    let cloned = config.clone();
    assert_eq!(config.max_requests, cloned.max_requests);
    assert_eq!(config.period_seconds, cloned.period_seconds);
    assert_eq!(config.burst_size, cloned.burst_size);
}

#[test]
fn test_config_new() {
    // This test will use environment variables or defaults
    let config = Config::new();

    // Verify that the config has been created with some values
    assert!(!config.credentials.username.is_empty());
    assert!(!config.credentials.api_key.is_empty());
    assert!(!config.rest_api.base_url.is_empty());
    assert!(config.rest_api.timeout > 0);
    assert!(!config.websocket.url.is_empty());
    assert!(config.websocket.reconnect_interval > 0);
    assert!(config.rate_limiter.max_requests > 0);
    assert!(config.rate_limiter.period_seconds > 0);
    assert!(config.sleep_hours > 0);
    // page_size can be 0 if TX_PAGE_SIZE env var is set to 0, so we just check it exists
    // The default should be 50, but we don't enforce it in the test
}

#[test]
fn test_config_default() {
    let config = Config::default();

    // Default should be the same as new()
    assert!(!config.credentials.username.is_empty());
    assert!(!config.rest_api.base_url.is_empty());
}

#[test]
fn test_config_clone() {
    let config = Config::new();
    let cloned = config.clone();

    assert_eq!(config.credentials.username, cloned.credentials.username);
    assert_eq!(config.rest_api.base_url, cloned.rest_api.base_url);
    assert_eq!(config.websocket.url, cloned.websocket.url);
    assert_eq!(config.sleep_hours, cloned.sleep_hours);
}

#[test]
fn test_config_serialization() {
    let config = Config {
        credentials: Credentials {
            username: "test".to_string(),
            password: "pass".to_string(),
            account_id: "acc".to_string(),
            api_key: "key".to_string(),
            client_token: None,
            account_token: None,
        },
        rest_api: RestApiConfig {
            base_url: "https://api.test.com".to_string(),
            timeout: 30,
        },
        websocket: WebSocketConfig {
            url: "wss://ws.test.com".to_string(),
            reconnect_interval: 5,
        },
        database: DatabaseConfig {
            url: "postgres://localhost/test".to_string(),
            max_connections: 5,
        },
        rate_limiter: RateLimiterConfig {
            max_requests: 10,
            period_seconds: 60,
            burst_size: 5,
        },
        sleep_hours: 1,
        page_size: 50,
        days_to_look_back: 30,
        api_version: Some(3),
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(
        config.credentials.username,
        deserialized.credentials.username
    );
    assert_eq!(config.rest_api.base_url, deserialized.rest_api.base_url);
    assert_eq!(config.sleep_hours, deserialized.sleep_hours);
}

#[test]
fn test_config_display() {
    let config = Config::new();
    let display = format!("{}", config);

    // DisplaySimple should produce some output
    assert!(!display.is_empty());
}

#[test]
fn test_credentials_display() {
    let creds = Credentials {
        username: "test_user".to_string(),
        password: "test_pass".to_string(),
        account_id: "ACC123".to_string(),
        api_key: "key123".to_string(),
        client_token: None,
        account_token: None,
    };

    let display = format!("{}", creds);
    assert!(!display.is_empty());
}

#[test]
fn test_rest_api_config_serialization() {
    let config = RestApiConfig {
        base_url: "https://api.example.com".to_string(),
        timeout: 45,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RestApiConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.base_url, deserialized.base_url);
    assert_eq!(config.timeout, deserialized.timeout);
}

#[test]
fn test_websocket_config_serialization() {
    let config = WebSocketConfig {
        url: "wss://ws.example.com".to_string(),
        reconnect_interval: 10,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: WebSocketConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.url, deserialized.url);
    assert_eq!(config.reconnect_interval, deserialized.reconnect_interval);
}

#[test]
fn test_rate_limiter_config_serialization() {
    let config = RateLimiterConfig {
        max_requests: 20,
        period_seconds: 120,
        burst_size: 10,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: RateLimiterConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.max_requests, deserialized.max_requests);
    assert_eq!(config.period_seconds, deserialized.period_seconds);
    assert_eq!(config.burst_size, deserialized.burst_size);
}

#[test]
fn test_config_api_version_none() {
    let mut config = Config::new();
    config.api_version = None;

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: Config = serde_json::from_str(&json).unwrap();

    assert_eq!(config.api_version, deserialized.api_version);
}

#[test]
fn test_config_api_version_some() {
    let mut config = Config::new();
    config.api_version = Some(2);

    assert_eq!(config.api_version, Some(2));

    config.api_version = Some(3);
    assert_eq!(config.api_version, Some(3));
}

#[test]
fn test_credentials_with_tokens() {
    let creds = Credentials {
        username: "test".to_string(),
        password: "pass".to_string(),
        account_id: "acc".to_string(),
        api_key: "key".to_string(),
        client_token: Some("client123".to_string()),
        account_token: Some("account456".to_string()),
    };

    assert_eq!(creds.client_token, Some("client123".to_string()));
    assert_eq!(creds.account_token, Some("account456".to_string()));
}

#[test]
fn test_credentials_without_tokens() {
    let creds = Credentials {
        username: "test".to_string(),
        password: "pass".to_string(),
        account_id: "acc".to_string(),
        api_key: "key".to_string(),
        client_token: None,
        account_token: None,
    };

    assert_eq!(creds.client_token, None);
    assert_eq!(creds.account_token, None);
}
