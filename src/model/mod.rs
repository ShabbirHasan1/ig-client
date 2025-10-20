/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
/// Authentication models and session management
pub mod auth;
/// HTTP request utilities with rate limiting and retry
pub mod http;
/// Request models for API calls
pub mod requests;
/// Response models from API calls
pub mod responses;
/// Retry configuration for HTTP requests
pub mod retry;
pub mod utils;
