use chrono::{Duration, Utc};
use ig_client::config::{Config, Credentials, RestApiConfig, WebSocketConfig};
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::{IgAuthenticator, IgSession, TokenTimer};
use ig_client::storage::config::DatabaseConfig;
use ig_client::utils::rate_limiter::RateLimitType;
use mockito::{self, Server};
use std::sync::{Arc, Mutex};
use tokio_test::block_on;

// Helper function to create a test config with mock server URL
fn create_test_config(server_url: &str) -> Config {
    Config {
        credentials: Credentials {
            username: "test_user".to_string(),
            password: "test_password".to_string(),
            api_key: "test_api_key".to_string(),
            account_id: "test_account".to_string(),
            client_token: Some("test_client_token".to_string()),
            account_token: Some("test_account_token".to_string()),
        },
        rest_api: RestApiConfig {
            base_url: server_url.to_string(),
            timeout: 30,
        },
        websocket: WebSocketConfig {
            url: "wss://example.com".to_string(),
            reconnect_interval: 5,
        },
        database: DatabaseConfig {
            url: "postgres://user:pass@localhost/ig_db".to_string(),
            max_connections: 5,
        },
        rate_limit_type: RateLimitType::NonTradingAccount,
        rate_limit_safety_margin: 0.8,
        sleep_hours: 1,
        page_size: 20,
        days_to_look_back: 7,
        api_version: Some(2),
    }
}

// Helper function to create a session with custom token timer
fn create_session_with_timer(timer: TokenTimer) -> IgSession {
    let mut session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Replace the token timer with our custom one
    session.token_timer = Arc::new(Mutex::new(timer));
    session
}

#[test]
fn test_relogin_with_valid_tokens() {
    let server = Server::new();

    // Create a session with fresh tokens (should not need relogin)
    let timer = TokenTimer::new();
    let session = create_session_with_timer(timer);

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call relogin - should return the same session without making API calls
    let result = block_on(auth.relogin(&session));

    assert!(result.is_ok(), "Relogin should succeed with valid tokens");
    let returned_session = result.unwrap();

    // Should return the same session data
    assert_eq!(returned_session.cst, session.cst);
    assert_eq!(returned_session.token, session.token);
    assert_eq!(returned_session.account_id, session.account_id);
}

#[test]
fn test_relogin_with_expired_tokens() {
    let mut server = Server::new();

    // Mock the login endpoint for relogin
    let mock = server.mock("POST", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "new_test_cst")
        .with_header("X-SECURITY-TOKEN", "new_test_token")
        .with_body(r#"{"clientId":"new_client","accountId":"test_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","oauthToken":null,"timezoneOffset":1}"#)
        .create();

    // Create a session with expired tokens
    let mut timer = TokenTimer::new();
    timer.expiry = Utc::now() - Duration::hours(1); // Expired 1 hour ago
    let session = create_session_with_timer(timer);

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call relogin - should perform actual login
    let result = block_on(auth.relogin(&session));

    assert!(result.is_ok(), "Relogin should succeed with expired tokens");
    let new_session = result.unwrap();

    // Should return new session with updated tokens
    assert_eq!(new_session.cst, "new_test_cst");
    assert_eq!(new_session.token, "new_test_token");
    assert_eq!(new_session.account_id, "test_account");

    mock.assert();
}

#[test]
fn test_relogin_with_tokens_near_expiry() {
    let mut server = Server::new();

    // Mock the login endpoint
    let mock = server.mock("POST", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "refreshed_cst")
        .with_header("X-SECURITY-TOKEN", "refreshed_token")
        .with_body(r#"{"clientId":"refreshed_client","accountId":"test_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","oauthToken":null,"timezoneOffset":1}"#)
        .create();

    // Create a session with tokens expiring in 15 minutes (within 30 minute margin)
    let mut timer = TokenTimer::new();
    timer.expiry = Utc::now() + Duration::minutes(15);
    let session = create_session_with_timer(timer);

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call relogin - should perform login due to margin
    let result = block_on(auth.relogin(&session));

    assert!(
        result.is_ok(),
        "Relogin should succeed with tokens near expiry"
    );
    let new_session = result.unwrap();

    // Should return new session
    assert_eq!(new_session.cst, "refreshed_cst");
    assert_eq!(new_session.token, "refreshed_token");

    mock.assert();
}

#[test]
fn test_relogin_with_max_age_exceeded() {
    let mut server = Server::new();

    // Mock the login endpoint
    let mock = server.mock("POST", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "max_age_refresh_cst")
        .with_header("X-SECURITY-TOKEN", "max_age_refresh_token")
        .with_body(r#"{"clientId":"max_age_client","accountId":"test_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","oauthToken":null,"timezoneOffset":1}"#)
        .create();

    // Create a session where max_age is exceeded but expiry is still valid
    let mut timer = TokenTimer::new();
    timer.expiry = Utc::now() + Duration::hours(2); // Still valid
    timer.max_age = Utc::now() - Duration::minutes(1); // Exceeded
    let session = create_session_with_timer(timer);

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call relogin - should perform login due to max_age
    let result = block_on(auth.relogin(&session));

    assert!(
        result.is_ok(),
        "Relogin should succeed when max_age is exceeded"
    );
    let new_session = result.unwrap();

    assert_eq!(new_session.cst, "max_age_refresh_cst");
    assert_eq!(new_session.token, "max_age_refresh_token");

    mock.assert();
}

#[test]
fn test_relogin_and_switch_account_success() {
    let mut server = Server::new();

    // Mock the switch account endpoint
    let mock = server.mock("PUT", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "switched_cst")
        .with_header("X-SECURITY-TOKEN", "switched_token")
        .with_body(r#"{"trailingStops":true,"dealingEnabled":true,"hasActiveDemoAccounts":true,"hasActiveLiveAccounts":true,"accountType":"SPREADBET","accountInfo":{"balance":10000.0,"deposit":0.0,"profitLoss":0.0,"available":10000.0},"currencySymbol":"£","currentAccountId":"target_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com"}"#)
        .create();

    // Create a session with valid tokens
    let timer = TokenTimer::new();
    let session = create_session_with_timer(timer);

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call relogin_and_switch_account
    let result = block_on(auth.relogin_and_switch_account(&session, "target_account", Some(false)));

    assert!(result.is_ok(), "Relogin and switch account should succeed");
    let new_session = result.unwrap();

    assert_eq!(new_session.account_id, "target_account");
    assert_eq!(new_session.cst, "switched_cst");
    assert_eq!(new_session.token, "switched_token");

    mock.assert();
}

#[test]
fn test_relogin_and_switch_account_with_expired_tokens() {
    let mut server = Server::new();

    // Mock the login endpoint first
    let login_mock = server.mock("POST", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "relogin_cst")
        .with_header("X-SECURITY-TOKEN", "relogin_token")
        .with_body(r#"{"clientId":"relogin_client","accountId":"original_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","oauthToken":null,"timezoneOffset":1}"#)
        .create();

    // Mock the switch account endpoint
    let switch_mock = server.mock("PUT", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "final_cst")
        .with_header("X-SECURITY-TOKEN", "final_token")
        .with_body(r#"{"trailingStops":true,"dealingEnabled":true,"hasActiveDemoAccounts":true,"hasActiveLiveAccounts":true,"accountType":"SPREADBET","accountInfo":{"balance":10000.0,"deposit":0.0,"profitLoss":0.0,"available":10000.0},"currencySymbol":"£","currentAccountId":"target_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com"}"#)
        .create();

    // Create a session with expired tokens
    let mut timer = TokenTimer::new();
    timer.expiry = Utc::now() - Duration::hours(1); // Expired
    let session = create_session_with_timer(timer);

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call relogin_and_switch_account - should first relogin, then switch
    let result = block_on(auth.relogin_and_switch_account(&session, "target_account", Some(true)));

    assert!(
        result.is_ok(),
        "Relogin and switch account should succeed with expired tokens"
    );
    let new_session = result.unwrap();

    assert_eq!(new_session.account_id, "target_account");
    assert_eq!(new_session.cst, "final_cst");
    assert_eq!(new_session.token, "final_token");

    login_mock.assert();
    switch_mock.assert();
}

#[test]
fn test_login_and_switch_account_uses_relogin() {
    let mut server = Server::new();

    // Mock the login endpoint
    let login_mock = server.mock("POST", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "login_cst")
        .with_header("X-SECURITY-TOKEN", "login_token")
        .with_body(r#"{"clientId":"login_client","accountId":"original_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","oauthToken":null,"timezoneOffset":1}"#)
        .create();

    // Mock the switch account endpoint
    let switch_mock = server.mock("PUT", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "switch_cst")
        .with_header("X-SECURITY-TOKEN", "switch_token")
        .with_body(r#"{"trailingStops":true,"dealingEnabled":true,"hasActiveDemoAccounts":true,"hasActiveLiveAccounts":true,"accountType":"SPREADBET","accountInfo":{"balance":10000.0,"deposit":0.0,"profitLoss":0.0,"available":10000.0},"currencySymbol":"£","currentAccountId":"target_account","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com"}"#)
        .create();

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call login_and_switch_account - should use the new relogin logic internally
    let result = block_on(auth.login_and_switch_account("target_account", Some(false)));

    assert!(result.is_ok(), "Login and switch account should succeed");
    let new_session = result.unwrap();

    assert_eq!(new_session.account_id, "target_account");
    assert_eq!(new_session.cst, "switch_cst");
    assert_eq!(new_session.token, "switch_token");

    login_mock.assert();
    switch_mock.assert();
}
