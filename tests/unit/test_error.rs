use ig_client::error::AppError;
use reqwest::StatusCode;

#[test]
fn test_app_error_display_rate_limit() {
    let error = AppError::RateLimitExceeded;
    assert_eq!(error.to_string(), "rate limit exceeded");
}

#[test]
fn test_app_error_display_unauthorized() {
    let error = AppError::Unauthorized;
    assert_eq!(error.to_string(), "unauthorized");
}

#[test]
fn test_app_error_display_not_found() {
    let error = AppError::NotFound;
    assert_eq!(error.to_string(), "not found");
}

#[test]
fn test_app_error_display_unexpected() {
    let error = AppError::Unexpected(StatusCode::BAD_REQUEST);
    assert!(error.to_string().contains("400"));
}

#[test]
fn test_app_error_display_serialization() {
    let error = AppError::SerializationError("Invalid format".to_string());
    assert_eq!(error.to_string(), "serialization error: Invalid format");
}

#[test]
fn test_app_error_display_websocket() {
    let error = AppError::WebSocketError("Connection closed".to_string());
    assert_eq!(error.to_string(), "websocket error: Connection closed");
}

#[test]
fn test_app_error_display_deserialization() {
    let error = AppError::Deserialization("Invalid JSON".to_string());
    assert_eq!(error.to_string(), "deserialization error: Invalid JSON");
}

#[test]
fn test_app_error_display_invalid_input() {
    let error = AppError::InvalidInput("Size must be positive".to_string());
    assert_eq!(error.to_string(), "invalid input: Size must be positive");
}

// Note: reqwest::Error cannot be easily constructed in tests
// This conversion is tested through integration tests

#[test]
fn test_app_error_from_serde() {
    let json = r#"{"invalid": json}"#;
    let serde_error = serde_json::from_str::<serde_json::Value>(json).unwrap_err();
    let app_error: AppError = serde_error.into();

    match app_error {
        AppError::Json(_) => (),
        _ => panic!("Expected Json error"),
    }
}

#[test]
fn test_app_error_from_io() {
    let io_error = std::io::Error::other("test");
    let app_error: AppError = io_error.into();

    match app_error {
        AppError::Io(_) => (),
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_app_error_oauth_token_expired() {
    let error = AppError::OAuthTokenExpired;
    assert_eq!(error.to_string(), "oauth token expired");
}
