/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/5/25
******************************************************************************/
use reqwest::StatusCode;
use std::fmt::{Display, Formatter};
use std::{fmt, io};

/// Error type for fetch operations
#[derive(Debug)]
pub enum FetchError {
    /// Network error from reqwest
    Reqwest(reqwest::Error),
    /// Database error from sqlx
    Sqlx(sqlx::Error),
    /// Error during parsing
    Parser(String),
}

impl Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Reqwest(e) => write!(f, "network error: {e}"),
            FetchError::Sqlx(e) => write!(f, "db error: {e}"),
            FetchError::Parser(msg) => write!(f, "parser error: {msg}"),
        }
    }
}

impl std::error::Error for FetchError {}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> Self {
        FetchError::Reqwest(err)
    }
}

impl From<sqlx::Error> for FetchError {
    fn from(err: sqlx::Error) -> Self {
        FetchError::Sqlx(err)
    }
}

/// Error type for authentication operations
#[derive(Debug)]
pub enum AuthError {
    /// Network error from reqwest
    Network(reqwest::Error),
    /// I/O error
    Io(io::Error),
    /// JSON serialization or deserialization error
    Json(serde_json::Error),
    /// Other unspecified error
    Other(String),
    /// Invalid credentials error
    BadCredentials,
    /// Unexpected HTTP status code
    Unexpected(StatusCode),
    /// Rate limit exceeded error
    RateLimitExceeded,
}

impl Display for AuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Network(e) => write!(f, "network error: {e}"),
            AuthError::Io(e) => write!(f, "io error: {e}"),
            AuthError::Json(e) => write!(f, "json error: {e}"),
            AuthError::Other(msg) => write!(f, "other error: {msg}"),
            AuthError::BadCredentials => write!(f, "bad credentials"),
            AuthError::Unexpected(s) => write!(f, "unexpected http status: {s}"),
            AuthError::RateLimitExceeded => write!(f, "rate limit exceeded"),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<reqwest::Error> for AuthError {
    fn from(e: reqwest::Error) -> Self {
        AuthError::Network(e)
    }
}
impl From<Box<dyn std::error::Error + Send + Sync>> for AuthError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        match e.downcast::<reqwest::Error>() {
            Ok(req) => AuthError::Network(*req),
            Err(e) => match e.downcast::<serde_json::Error>() {
                Ok(js) => AuthError::Json(*js),
                Err(e) => match e.downcast::<std::io::Error>() {
                    Ok(ioe) => AuthError::Io(*ioe),
                    Err(other) => AuthError::Other(other.to_string()),
                },
            },
        }
    }
}
impl From<Box<dyn std::error::Error>> for AuthError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        match e.downcast::<reqwest::Error>() {
            Ok(req) => AuthError::Network(*req),
            Err(e) => match e.downcast::<serde_json::Error>() {
                Ok(js) => AuthError::Json(*js),
                Err(e) => match e.downcast::<io::Error>() {
                    Ok(ioe) => AuthError::Io(*ioe),
                    Err(other) => AuthError::Other(other.to_string()),
                },
            },
        }
    }
}
impl From<AppError> for AuthError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Network(e) => AuthError::Network(e),
            AppError::Io(e) => AuthError::Io(e),
            AppError::Json(e) => AuthError::Json(e),
            AppError::Unexpected(s) => AuthError::Unexpected(s),
            _ => AuthError::Other("unknown error".to_string()),
        }
    }
}

/// General application error type
#[derive(Debug)]
pub enum AppError {
    /// Network error from reqwest
    Network(reqwest::Error),
    /// I/O error
    Io(io::Error),
    /// JSON serialization or deserialization error
    Json(serde_json::Error),
    /// Unexpected HTTP status code
    Unexpected(StatusCode),
    /// Database error from sqlx
    Db(sqlx::Error),
    /// Unauthorized access error
    Unauthorized,
    /// OAuth token expired error (requires token refresh)
    OAuthTokenExpired,
    /// Resource not found error
    NotFound,
    /// API rate limit exceeded
    RateLimitExceeded,
    /// Error during serialization or deserialization
    SerializationError(String),
    /// WebSocket communication error
    WebSocketError(String),
    /// Deserialization error with details
    Deserialization(String),
    /// Represents an error type for invalid input.
    ///
    /// This enum variant is used to indicate that invalid input has been provided,
    /// typically taking the form of a string that describes the nature of the issue.
    ///
    /// # Variants
    /// * `InvalidInput(String)`
    ///   - Contains a `String` value that provides more details about why the input
    ///     was considered invalid, such as specific formatting issues or constraints
    ///     that were violated.
    ///
    InvalidInput(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Network(e) => write!(f, "network error: {e}"),
            AppError::Io(e) => write!(f, "io error: {e}"),
            AppError::Json(e) => write!(f, "json error: {e}"),
            AppError::Unexpected(s) => write!(f, "unexpected http status: {s}"),
            AppError::Db(e) => write!(f, "db error: {e}"),
            AppError::Unauthorized => write!(f, "unauthorized"),
            AppError::OAuthTokenExpired => write!(f, "oauth token expired"),
            AppError::NotFound => write!(f, "not found"),
            AppError::RateLimitExceeded => write!(f, "rate limit exceeded"),
            AppError::SerializationError(s) => write!(f, "serialization error: {s}"),
            AppError::WebSocketError(s) => write!(f, "websocket error: {s}"),
            AppError::Deserialization(s) => write!(f, "deserialization error: {s}"),
            AppError::InvalidInput(s) => write!(f, "invalid input: {s}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Network(e)
    }
}
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self {
        AppError::Io(e)
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e)
    }
}
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Db(e)
    }
}
impl From<AuthError> for AppError {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::Network(e) => AppError::Network(e),
            AuthError::Io(e) => AppError::Io(e),
            AuthError::Json(e) => AppError::Json(e),
            AuthError::BadCredentials => AppError::Unauthorized,
            AuthError::Unexpected(s) => AppError::Unexpected(s),
            _ => AppError::Unexpected(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        match e.downcast::<reqwest::Error>() {
            Ok(req) => AppError::Network(*req),
            Err(e) => match e.downcast::<serde_json::Error>() {
                Ok(js) => AppError::Json(*js),
                Err(e) => match e.downcast::<std::io::Error>() {
                    Ok(ioe) => AppError::Io(*ioe),
                    Err(_) => AppError::Unexpected(StatusCode::INTERNAL_SERVER_ERROR),
                },
            },
        }
    }
}
