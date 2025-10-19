use ig_client::config::{Config, Credentials, RestApiConfig, WebSocketConfig};
use ig_client::error::AppError;
use ig_client::session::interface::IgSession;
use ig_client::storage::config::DatabaseConfig;
use ig_client::transport::http_client::{IgHttpClient, IgHttpClientImpl};
use ig_client::utils::rate_limiter::RateLimitType;
use mockito::{self, Server};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_test::block_on;

fn create_test_config(base_url: &str) -> Arc<Config> {
    Arc::new(Config {
        credentials: Credentials {
            username: "test_user".to_string(),
            password: "test_password".to_string(),
            api_key: "test_api_key".to_string(),
            account_id: "test_account".to_string(),
            client_token: Some("test_client_token".to_string()),
            account_token: Some("test_account_token".to_string()),
        },
        rest_api: RestApiConfig {
            base_url: base_url.to_string(),
            timeout: 5,
        },
        websocket: WebSocketConfig {
            url: "wss://example.com".to_string(),
            reconnect_interval: 5,
        },
        rate_limit_type: RateLimitType::NonTradingAccount,
        rate_limit_safety_margin: 0.8,
        database: DatabaseConfig {
            url: "postgres://user:pass@localhost/ig_db".to_string(),
            max_connections: 5,
        },
        api_version: Some(2),
        sleep_hours: 1,
        page_size: 20,
        days_to_look_back: 7,
    })
}

fn create_test_session() -> IgSession {
    IgSession::new(
        "test_cst".to_string(),
        "test_xst".to_string(),
        "test_account".to_string(),
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct TestRequest {
    name: String,
    value: i32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct TestResponse {
    result: String,
    code: i32,
}

#[test]
fn test_http_client_new() {
    let config = create_test_config("https://demo-api.ig.com");
    let _client = IgHttpClientImpl::new(config.clone());

    // Verify client was created successfully
    assert!(!config.rest_api.base_url.is_empty());
    assert_eq!(config.rest_api.timeout, 5);
}

// We cannot test build_url directly as it is a private method

#[test]
fn test_request_with_mockito() {
    // This test uses mockito to mock HTTP responses
    let mut server = Server::new();

    // Create test config with mockito server URL
    let config = create_test_config(&server.url());
    let client = IgHttpClientImpl::new(config.clone());
    let session = create_test_session();

    // Mock a successful response
    let mock = server
        .mock("GET", "/test")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"result":"success","code":200}"#)
        .create();

    // Make the request
    let response: Result<TestResponse, AppError> =
        block_on(client.request(Method::GET, "test", &session, None::<&TestRequest>, "1"));

    // Verify the response
    assert!(response.is_ok());
    let test_response = response.unwrap();
    assert_eq!(test_response.result, "success");
    assert_eq!(test_response.code, 200);

    // Verify the mock was called
    mock.assert();
}

#[test]
fn test_request_no_auth_with_mockito() {
    // This test uses mockito to mock HTTP responses for unauthenticated requests
    let mut server = Server::new();

    // Create test config with mockito server URL
    let config = create_test_config(&server.url());
    let client = IgHttpClientImpl::new(config.clone());

    // Mock a successful response
    let mock = server
        .mock("POST", "/login")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"result":"logged_in","code":200}"#)
        .create();

    // Create a test request body
    let request = TestRequest {
        name: "test_user".to_string(),
        value: 123,
    };

    // Make the request
    let response: Result<TestResponse, AppError> =
        block_on(client.request_no_auth(Method::POST, "login", Some(&request), "1"));

    // Verify the response
    assert!(response.is_ok());
    let test_response = response.unwrap();
    assert_eq!(test_response.result, "logged_in");
    assert_eq!(test_response.code, 200);

    // Verify the mock was called
    mock.assert();
}
