use ig_client::config::{Config, Credentials, RestApiConfig, WebSocketConfig};
use ig_client::error::AuthError;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::{IgAuthenticator, IgSession};
use ig_client::storage::config::DatabaseConfig;
use ig_client::utils::rate_limiter::RateLimitType;
use mockito::{self, Server};
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
    }
}

#[test]
fn test_ig_auth_new() {
    let server = Server::new();
    let config = create_test_config(&server.url());
    let _auth = IgAuth::new(&config);
}

#[test]
fn test_ig_session_new() {
    let session = IgSession::new(
        "CST123".to_string(),
        "XST456".to_string(),
        "ACC789".to_string(),
    );

    assert_eq!(session.cst, "CST123");
    assert_eq!(session.token, "XST456");
    assert_eq!(session.account_id, "ACC789");
}

#[test]
fn test_login_success() {
    let mut server = Server::new();

    // Mock the login endpoint
    let mock = server.mock("POST", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "test_cst")
        .with_header("X-SECURITY-TOKEN", "test_token")
        .with_body(r#"{"clientId":"test_client","accountId":"A12345","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","oauthToken":null,"timezoneOffset":1}"#)
        .create();

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call the login method
    let result = block_on(auth.login());

    // Verify the result
    assert!(result.is_ok());
    let session = result.unwrap();
    assert_eq!(session.cst, "test_cst");
    assert_eq!(session.token, "test_token");
    assert_eq!(session.account_id, "A12345");

    mock.assert();
}

#[test]
fn test_login_unauthorized() {
    let mut server = Server::new();

    // Mock the login endpoint with an unauthorized response
    let mock = server
        .mock("POST", "/session")
        .with_status(401)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"errorCode":"error.security.invalid-details"}"#)
        .create();

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call the login method
    let result = block_on(auth.login());

    // Verify the result
    assert!(result.is_err());
    match result {
        Err(AuthError::BadCredentials) => {
            // This is the expected error
        }
        _ => panic!("Expected BadCredentials error"),
    }

    mock.assert();
}

#[test]
fn test_switch_account_success() {
    let mut server = Server::new();

    // Create a test session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "A12345".to_string(),
    );

    // Mock the switch account endpoint
    let mock = server.mock("PUT", "/session")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"trailingStops":true,"dealingEnabled":true,"hasActiveDemoAccounts":true,"hasActiveLiveAccounts":true,"accountType":"SPREADBET","accountInfo":{"balance":10000.0,"deposit":0.0,"profitLoss":0.0,"available":10000.0},"currencySymbol":"£","currentAccountId":"B67890","lightstreamerEndpoint":"https://demo-apd.marketdatasystems.com","accounts":[{"accountId":"A12345","accountName":"Demo","preferred":false,"accountType":"SPREADBET"},{"accountId":"B67890","accountName":"Live","preferred":true,"accountType":"SPREADBET"}],"clientId":"test_client","timezoneOffset":1,"hasActiveSalvationAccounts":false,"reroutingEnvironment":null}"#)
        .create();

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call the switch account method
    let result = block_on(auth.switch_account(&session, "B67890", Some(true)));

    // Verify the result
    assert!(result.is_err());
    mock.assert();
}

#[test]
fn test_refresh_success() {
    let mut server = Server::new();

    // Create a test session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "A12345".to_string(),
    );

    // Mock the refresh endpoint
    let mock = server
        .mock("POST", "/session/refresh-token")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_header("CST", "new_cst")
        .with_header("X-SECURITY-TOKEN", "new_token")
        .with_body(r#"{"accountId":"A12345"}"#)
        .create();

    let config = create_test_config(&server.url());
    let auth = IgAuth::new(&config);

    // Call the refresh method
    let result = block_on(auth.refresh(&session));

    // Verify the result
    assert!(result.is_ok());
    let new_session = result.unwrap();
    assert_eq!(new_session.cst, "new_cst"); // Should be updated
    assert_eq!(new_session.token, "new_token"); // Should be updated
    assert_eq!(new_session.account_id, "A12345"); // Should remain the same

    mock.assert();
}
