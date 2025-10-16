use ig_client::application::models::order::{
    ClosePositionRequest, CreateOrderRequest, Direction, OrderConfirmation, OrderType, Status,
    TimeInForce, UpdatePositionRequest,
};
use ig_client::application::models::working_order::{
    CreateWorkingOrderRequest, CreateWorkingOrderResponse,
};
use ig_client::application::services::order_service::OrderServiceImpl;
use ig_client::config::Config;
use ig_client::error::AppError;
use ig_client::session::interface::IgSession;
use ig_client::transport::http_client::IgHttpClient;
use ig_client::utils::rate_limiter::RateLimitType;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::sync::Arc;

// Mock HTTP client for testing service methods without actual network calls
struct MockHttpClient {}

#[async_trait::async_trait]
impl IgHttpClient for MockHttpClient {
    async fn request<T: serde::Serialize + Sync, R: DeserializeOwned>(
        &self,
        _method: Method,
        _path: &str,
        _session: &IgSession,
        _body: Option<&T>,
        _version: &str,
    ) -> Result<R, AppError> {
        // This mock will never be called in our tests
        // We're only testing validation logic that happens before network calls
        panic!("Mock HTTP client should not be called in these tests");
    }

    async fn request_no_auth<T: serde::Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        _method: Method,
        _path: &str,
        _body: Option<&T>,
        _version: &str,
    ) -> Result<R, AppError> {
        // This mock will never be called in our tests
        panic!("Mock HTTP client should not be called in these tests");
    }
}

#[test]
fn test_create_order_request_market() {
    // Test the market constructor of CreateOrderRequest
    let order = CreateOrderRequest::market(
        "OP.D.OTCDAX1.021100P.IP".to_string(),
        Direction::Buy,
        1.0,
        None,
    );

    // Verify that the fields were set correctly
    assert_eq!(order.epic, "OP.D.OTCDAX1.021100P.IP");
    assert!(matches!(order.direction, Direction::Buy));
    assert_eq!(order.size, 1.0);
    assert!(matches!(order.order_type, OrderType::Market));
    assert!(matches!(order.time_in_force, TimeInForce::FillOrKill));
}

#[test]
fn test_create_order_request_limit() {
    // Test the limit constructor of CreateOrderRequest
    let order = CreateOrderRequest::limit(
        "OP.D.OTCDAX1.021100P.IP".to_string(),
        Direction::Sell,
        1.0,
        1.2345,
        None,
    );

    // Verify that the fields were set correctly
    assert_eq!(order.epic, "OP.D.OTCDAX1.021100P.IP");
    assert!(matches!(order.direction, Direction::Sell));
    assert_eq!(order.size, 1.0);
    assert!(matches!(order.order_type, OrderType::Limit));
    assert!(matches!(
        order.time_in_force,
        TimeInForce::GoodTillCancelled
    ));
    assert_eq!(order.level, Some(1.2345));
}

#[test]
fn test_create_order_request_with_reference() {
    // Test the with_reference method
    let order = CreateOrderRequest::market(
        "OP.D.OTCDAX1.021100P.IP".to_string(),
        Direction::Buy,
        1.0,
        None,
    )
    .with_reference("TEST_REF".to_string());

    // Verify deal_reference field is set correctly
    assert_eq!(order.deal_reference, Some("TEST_REF".to_string()));
}

#[test]
fn test_create_order_request_with_stop_loss() {
    // Test the with_stop_loss method
    let order = CreateOrderRequest::market(
        "OP.D.OTCDAX1.021100P.IP".to_string(),
        Direction::Buy,
        1.0,
        None,
    )
    .with_stop_loss(1.2000);

    // Verify stop_level field is set correctly
    assert_eq!(order.stop_level, Some(1.2000));
}

#[test]
fn test_create_order_request_with_take_profit() {
    // Test the with_take_profit method
    let order = CreateOrderRequest::market(
        "OP.D.OTCDAX1.021100P.IP".to_string(),
        Direction::Buy,
        1.0,
        None,
    )
    .with_take_profit(1.3000);

    // Verify limit_level field is set correctly
    assert_eq!(order.limit_level, Some(1.3000));
}

#[test]
fn test_close_position_request_market() {
    // Test the market constructor of ClosePositionRequest
    let request = ClosePositionRequest::market("DEAL123".to_string(), Direction::Sell, 1.0);

    // Verify that the fields were set correctly
    assert!(matches!(request.direction, Direction::Sell));
    assert_eq!(request.size, 1.0);
    assert!(matches!(request.order_type, OrderType::Market));
    assert!(matches!(request.time_in_force, TimeInForce::FillOrKill));
    assert_eq!(request.level, None);
}

#[test]
fn test_update_position_request() {
    // Create an update request
    let update = UpdatePositionRequest {
        stop_level: Some(1.2000),
        limit_level: Some(1.3000),
        trailing_stop: Some(false),
        trailing_stop_distance: Some(0.01),
    };

    // Verify that the fields were set correctly
    assert_eq!(update.stop_level, Some(1.2000));
    assert_eq!(update.limit_level, Some(1.3000));
    assert_eq!(update.trailing_stop, Some(false));
    assert_eq!(update.trailing_stop_distance, Some(0.01));
}

#[test]
fn test_order_service_config() {
    // Create a mock session
    let session = IgSession::new(
        "CST123".to_string(),
        "XST123".to_string(),
        "ACC123".to_string(),
    );

    // Verify session was created successfully
    assert_eq!(session.cst, "CST123");
    assert_eq!(session.token, "XST123");
    assert_eq!(session.account_id, "ACC123");

    // Test service config methods
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    let client = Arc::new(MockHttpClient {});
    let mut service = OrderServiceImpl::new(config.clone(), client);

    // Test get_config
    assert!(Arc::ptr_eq(&service.get_config(), &config));

    // Test set_config
    let new_config = Arc::new(Config::default());
    service.set_config(new_config.clone());
    assert!(Arc::ptr_eq(&service.get_config(), &new_config));
}

#[test]
fn test_create_working_order_request() {
    // Create a working order request using the builder pattern
    let request =
        CreateWorkingOrderRequest::limit("EPIC123".to_string(), Direction::Buy, 1.0, 100.0)
            .with_reference("TEST_REF".to_string())
            .with_expiry("DFB".to_string());

    // Verify fields
    assert_eq!(request.epic, "EPIC123");
    assert_eq!(request.expiry, "DFB");
    assert!(matches!(request.direction, Direction::Buy));
    assert_eq!(request.size, 1.0);
    assert_eq!(request.level, 100.0);
    assert!(matches!(request.order_type, OrderType::Limit));
    assert!(matches!(
        request.time_in_force,
        TimeInForce::GoodTillCancelled
    ));
    assert_eq!(request.deal_reference, Some("TEST_REF".to_string()));
}

#[test]
fn test_create_working_order_response() {
    // Create a working order response
    let response = CreateWorkingOrderResponse {
        deal_reference: "DEAL123".to_string(),
    };

    // Verify fields
    assert_eq!(response.deal_reference, "DEAL123");
}

#[test]
fn test_order_confirmation() {
    // Create an order confirmation
    let confirmation = OrderConfirmation {
        date: "2023-01-01".to_string(),
        status: Status::Accepted,
        reason: None,
        deal_status: Some("ACCEPTED".to_string()),
        epic: Some("EPIC123".to_string()),
        expiry: Some("DFB".to_string()),
        deal_reference: "DEAL123".to_string(),
        deal_id: Some("ID123".to_string()),
        level: Some(100.0),
        size: Some(1.0),
        direction: Some(Direction::Buy),
        stop_level: None,
        limit_level: None,
        stop_distance: None,
        limit_distance: None,
        guaranteed_stop: Some(false),
        trailing_stop: Some(false),
    };

    // Verify fields
    assert_eq!(confirmation.date, "2023-01-01");
    assert!(matches!(confirmation.status, Status::Accepted));
    assert_eq!(confirmation.deal_status, Some("ACCEPTED".to_string()));
    assert_eq!(confirmation.epic, Some("EPIC123".to_string()));
    assert_eq!(confirmation.deal_reference, "DEAL123");
    assert_eq!(confirmation.deal_id, Some("ID123".to_string()));
    assert_eq!(confirmation.level, Some(100.0));
    assert_eq!(confirmation.size, Some(1.0));
    assert!(matches!(confirmation.direction, Some(Direction::Buy)));
}
