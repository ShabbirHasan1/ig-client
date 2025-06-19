use ig_client::error::{AppError, AuthError, FetchError};
use reqwest::StatusCode;
use serde_json::Error as JsonError;
use sqlx::Error as SqlxError;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{Error as IoError, ErrorKind};

// Custom error type for testing
#[derive(Debug)]
struct TestError(String);

impl Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError: {}", self.0)
    }
}

impl Error for TestError {}

// Helper function to test Display implementation
fn assert_display_contains<T: Display>(value: &T, expected: &str) {
    let display_string = value.to_string();
    assert!(
        display_string.contains(expected),
        "Expected '{}' to contain '{}', but it didn't",
        display_string,
        expected
    );
}

#[test]
fn test_app_error_from_io_error() {
    let io_error = IoError::new(ErrorKind::NotFound, "file not found");
    let app_error = AppError::from(io_error);

    match app_error {
        AppError::Io(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Io, got {:?}", app_error),
    }

    assert_display_contains(&app_error, "io error");
}

#[test]
fn test_app_error_from_serde_json_error() {
    // Create a JSON error
    let json_str = r#"{"invalid": json"#;
    let json_error: JsonError = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();

    let app_error = AppError::from(json_error);

    match app_error {
        AppError::Json(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Json, got {:?}", app_error),
    }

    assert_display_contains(&app_error, "json error");
}

#[test]
fn test_app_error_from_sqlx_error() {
    // Create a SqlxError (using a simple variant since we can't easily create a real one)
    let sqlx_error = SqlxError::RowNotFound;

    let app_error = AppError::from(sqlx_error);

    match app_error {
        AppError::Db(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Db, got {:?}", app_error),
    }

    assert_display_contains(&app_error, "db error");
}

#[test]
fn test_app_error_from_auth_error() {
    // Create an AuthError
    let auth_error = AuthError::BadCredentials;

    let app_error = AppError::from(auth_error);

    match app_error {
        AppError::Unauthorized => {
            // Test passed
        }
        _ => panic!("Expected AppError::Unauthorized, got {:?}", app_error),
    }

    assert_display_contains(&app_error, "unauthorized");

    // Test another variant
    let auth_error = AuthError::Unexpected(StatusCode::INTERNAL_SERVER_ERROR);
    let app_error = AppError::from(auth_error);

    match app_error {
        AppError::Unexpected(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Unexpected, got {:?}", app_error),
    }
}

#[test]
fn test_app_error_unauthorized() {
    let app_error = AppError::Unauthorized;
    assert_display_contains(&app_error, "unauthorized");
}

#[test]
fn test_app_error_not_found() {
    let app_error = AppError::NotFound;
    assert_display_contains(&app_error, "not found");
}

#[test]
fn test_app_error_rate_limit_exceeded() {
    let app_error = AppError::RateLimitExceeded;
    assert_display_contains(&app_error, "rate limit exceeded");
}

#[test]
fn test_app_error_serialization_error() {
    let app_error = AppError::SerializationError("test error".to_string());
    assert_display_contains(&app_error, "serialization error");
    assert_display_contains(&app_error, "test error");
}

#[test]
fn test_app_error_websocket_error() {
    let app_error = AppError::WebSocketError("connection closed".to_string());
    assert_display_contains(&app_error, "websocket error");
    assert_display_contains(&app_error, "connection closed");
}

#[test]
fn test_app_error_unexpected() {
    let app_error = AppError::Unexpected(StatusCode::BAD_REQUEST);
    assert_display_contains(&app_error, "unexpected http status");
    assert_display_contains(&app_error, "400");
}

#[test]
fn test_app_error_invalid_input() {
    let app_error = AppError::InvalidInput("invalid parameter".to_string());
    assert_display_contains(&app_error, "invalid input");
    assert_display_contains(&app_error, "invalid parameter");
}

#[test]
fn test_app_error_deserialization() {
    let app_error = AppError::Deserialization("failed to deserialize".to_string());
    assert_display_contains(&app_error, "deserialization error");
    assert_display_contains(&app_error, "failed to deserialize");
}

#[test]
fn test_app_error_from_box_dyn_error() {
    // Create a Box<dyn Error> containing an IoError
    let io_error = IoError::new(ErrorKind::NotFound, "file not found");
    let boxed_error: Box<dyn Error> = Box::new(io_error);

    // Convert to AppError
    let app_error = AppError::from(boxed_error);

    match app_error {
        AppError::Io(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Io, got {:?}", app_error),
    }

    // Create a Box<dyn Error> containing a JsonError
    let json_str = r#"{"invalid": json"#;
    let json_error: JsonError = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let boxed_error: Box<dyn Error> = Box::new(json_error);

    // Convert to AppError
    let app_error = AppError::from(boxed_error);

    match app_error {
        AppError::Json(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Json, got {:?}", app_error),
    }

    // Create a Box<dyn Error> containing a different error type
    let boxed_error: Box<dyn Error> = Box::new(TestError("test error".to_string()));

    // Convert to AppError - should default to Unexpected
    let app_error = AppError::from(boxed_error);

    match app_error {
        AppError::Unexpected(_) => {
            // Test passed
        }
        _ => panic!("Expected AppError::Unexpected, got {:?}", app_error),
    }
}

#[test]
fn test_auth_error_display() {
    let auth_error = AuthError::BadCredentials;
    assert_display_contains(&auth_error, "bad credentials");

    // We create a reqwest error indirectly since from_static is not available
    let auth_error = AuthError::Other("network error".to_string());
    assert_display_contains(&auth_error, "network error");

    let auth_error = AuthError::Other("custom error".to_string());
    assert_display_contains(&auth_error, "other error");
    assert_display_contains(&auth_error, "custom error");
}

#[test]
fn test_auth_error_from_box_dyn_error() {
    // Create a Box<dyn Error> containing an IoError
    let io_error = IoError::new(ErrorKind::NotFound, "file not found");
    let boxed_error: Box<dyn Error> = Box::new(io_error);

    // Convert to AuthError
    let auth_error = AuthError::from(boxed_error);

    match auth_error {
        AuthError::Io(_) => {
            // Test passed
        }
        _ => panic!("Expected AuthError::Io, got {:?}", auth_error),
    }

    // Create a Box<dyn Error> containing a JsonError
    let json_str = r#"{"invalid": json"#;
    let json_error: JsonError = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let boxed_error: Box<dyn Error> = Box::new(json_error);

    // Convert to AuthError
    let auth_error = AuthError::from(boxed_error);

    match auth_error {
        AuthError::Json(_) => {
            // Test passed
        }
        _ => panic!("Expected AuthError::Json, got {:?}", auth_error),
    }

    // Create a Box<dyn Error> containing a different error type
    let boxed_error: Box<dyn Error> = Box::new(TestError("test error".to_string()));

    // Convert to AuthError - should be Other
    let auth_error = AuthError::from(boxed_error);

    match auth_error {
        AuthError::Other(_) => {
            // Test passed
        }
        _ => panic!("Expected AuthError::Other, got {:?}", auth_error),
    }
}

#[test]
fn test_auth_error_from_box_dyn_error_send_sync() {
    // We can't easily create a Box<dyn Error + Send + Sync> with reqwest::Error
    // since it would require an actual HTTP request, but we can test the other paths

    // Create a Box<dyn Error + Send + Sync> containing an IoError
    let io_error = IoError::new(ErrorKind::NotFound, "file not found");
    let boxed_error: Box<dyn Error + Send + Sync> = Box::new(io_error);

    // Convert to AuthError
    let auth_error = AuthError::from(boxed_error);

    match auth_error {
        AuthError::Io(_) => {
            // Test passed
        }
        _ => panic!("Expected AuthError::Io, got {:?}", auth_error),
    }

    // Create a Box<dyn Error + Send + Sync> containing a JsonError
    let json_str = r#"{"invalid": json"#;
    let json_error: JsonError = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
    let boxed_error: Box<dyn Error + Send + Sync> = Box::new(json_error);

    // Convert to AuthError
    let auth_error = AuthError::from(boxed_error);

    match auth_error {
        AuthError::Json(_) => {
            // Test passed
        }
        _ => panic!("Expected AuthError::Json, got {:?}", auth_error),
    }

    // Create a Box<dyn Error + Send + Sync> containing a different error type
    let boxed_error: Box<dyn Error + Send + Sync> = Box::new(TestError("test error".to_string()));

    // Convert to AuthError - should be Other
    let auth_error = AuthError::from(boxed_error);

    match auth_error {
        AuthError::Other(_) => {
            // Test passed
        }
        _ => panic!("Expected AuthError::Other, got {:?}", auth_error),
    }
}

#[test]
fn test_fetch_error_display() {
    let fetch_error = FetchError::Parser("parsing failed".to_string());
    assert_display_contains(&fetch_error, "parser error");
    assert_display_contains(&fetch_error, "parsing failed");

    let fetch_error = FetchError::Sqlx(SqlxError::RowNotFound);
    assert_display_contains(&fetch_error, "db error");
}

#[test]
fn test_auth_error_rate_limit_exceeded() {
    let auth_error = AuthError::RateLimitExceeded;
    assert_display_contains(&auth_error, "rate limit exceeded");
}

#[test]
fn test_auth_error_unexpected() {
    let auth_error = AuthError::Unexpected(StatusCode::BAD_REQUEST);
    assert_display_contains(&auth_error, "unexpected http status");
    assert_display_contains(&auth_error, "400");
}
