/// Helper utilities for managing session and automatic token refresh
///
/// This module provides utilities to automatically handle OAuth token refresh
/// when making API calls that might fail due to expired tokens.
use crate::error::AppError;
use crate::session::auth::IgAuth;
use crate::session::interface::{IgAuthenticator, IgSession};
use std::future::Future;
use tracing::{debug, info, warn};

/// Executes an async operation with automatic OAuth token refresh on expiration
///
/// This function wraps an async operation and automatically handles OAuth token
/// expiration by refreshing the token and retrying the operation once.
///
/// # Arguments
/// * `session` - Mutable reference to the current session
/// * `auth` - Reference to the authenticator for token refresh
/// * `operation` - Async closure that performs the API operation
///
/// # Returns
/// * `Result<T, AppError>` - The result of the operation or an error
///
/// # Examples
/// ```ignore
/// use ig_client::utils::session_helper::with_auto_refresh;
///
/// let mut session = auth.login().await?;
/// let result = with_auto_refresh(
///     &mut session,
///     &auth,
///     |s| async move { market_service.get_market_details(s, "EPIC").await }
/// ).await?;
/// ```
pub async fn with_auto_refresh<F, Fut, T>(
    session: &mut IgSession,
    auth: &IgAuth<'_>,
    operation: F,
) -> Result<T, AppError>
where
    F: for<'a> Fn(&'a IgSession) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    // Check if token needs refresh before attempting the operation
    if session.needs_token_refresh(Some(300)) {
        info!("OAuth token needs refresh - refreshing proactively");
        match auth.refresh(session).await {
            Ok(new_session) => {
                *session = new_session;
                debug!("Token refreshed successfully");
            }
            Err(e) => {
                warn!("Failed to refresh token proactively: {:?}", e);
                // Continue anyway - the operation might still work
            }
        }
    }

    // Try the operation
    match operation(session).await {
        Ok(result) => Ok(result),
        Err(AppError::OAuthTokenExpired) => {
            info!("OAuth token expired during operation - attempting refresh and retry");

            // Refresh the token
            match auth.refresh(session).await {
                Ok(new_session) => {
                    *session = new_session;
                    debug!("Token refreshed successfully after expiration");

                    // Retry the operation with the new token
                    operation(session).await
                }
                Err(e) => {
                    warn!("Failed to refresh expired token: {:?}", e);
                    Err(AppError::Unauthorized)
                }
            }
        }
        Err(e) => Err(e),
    }
}

/// Checks if a session needs token refresh and refreshes it if necessary
///
/// This is a simpler alternative to `with_auto_refresh` that just handles
/// the token refresh without wrapping an operation.
///
/// # Arguments
/// * `session` - Mutable reference to the current session
/// * `auth` - Reference to the authenticator for token refresh
/// * `margin_seconds` - Optional safety margin in seconds (default: 300 = 5 minutes)
///
/// # Returns
/// * `Result<bool, AppError>` - `true` if token was refreshed, `false` if not needed
///
/// # Examples
/// ```ignore
/// use ig_client::utils::session_helper::refresh_if_needed;
///
/// let mut session = auth.login().await?;
/// if refresh_if_needed(&mut session, &auth, Some(300)).await? {
///     println!("Token was refreshed");
/// }
/// ```
pub async fn refresh_if_needed(
    session: &mut IgSession,
    auth: &IgAuth<'_>,
    margin_seconds: Option<i64>,
) -> Result<bool, AppError> {
    if session.needs_token_refresh(margin_seconds) {
        info!("OAuth token needs refresh");
        match auth.refresh(session).await {
            Ok(new_session) => {
                *session = new_session;
                debug!("Token refreshed successfully");
                Ok(true)
            }
            Err(e) => {
                warn!("Failed to refresh token: {:?}", e);
                Err(AppError::Unauthorized)
            }
        }
    } else {
        Ok(false)
    }
}
