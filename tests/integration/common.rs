// Common utilities for integration tests

use ig_client::utils::logger::setup_logger;
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    config::Config,
    session::auth::IgAuth,
    session::interface::{IgAuthenticator, IgSession},
    transport::http_client::IgHttpClientImpl,
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::info;

/// Creates a test configuration
pub fn create_test_config() -> Arc<Config> {
    // Use the default configuration which should load from environment variables
    Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ))
}

/// Creates an HTTP client for tests
pub fn create_test_client(config: Arc<Config>) -> Arc<IgHttpClientImpl> {
    Arc::new(IgHttpClientImpl::new(config))
}

/// Creates an authenticator for tests
pub fn create_test_auth(config: &Config) -> IgAuth<'_> {
    IgAuth::new(config)
}

/// Performs login and optionally switches to the account specified in the config
pub fn login_with_account_switch() -> IgSession {
    setup_logger();
    let config = create_test_config();
    let _auth = create_test_auth(&config);

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Login and get a session
    rt.block_on(async {
        login_with_account_switch_async()
            .await
            .expect("Failed to login")
    })
}

/// Async version of login_with_account_switch for use in async tests
/// Returns a Result with the session or an error message
pub async fn login_with_account_switch_async() -> Result<IgSession, String> {
    setup_logger();
    let config = create_test_config();
    let auth = create_test_auth(&config);

    // Login and get a session
    let session = match auth.login().await {
        Ok(session) => session,
        Err(e) => {
            return Err(format!("Failed to login: {e:?}"));
        }
    };

    // Check if we need to switch accounts
    if !config.credentials.account_id.is_empty()
        && session.account_id != config.credentials.account_id
    {
        info!("Switching to account: {}", config.credentials.account_id);
        match auth
            .switch_account(&session, &config.credentials.account_id, Some(true))
            .await
        {
            Ok(new_session) => {
                info!("Switched to account: {}", new_session.account_id);
                Ok(new_session)
            }
            Err(e) => {
                info!(
                    "Could not switch to account {}: {:?}. Continuing with current account.",
                    config.credentials.account_id, e
                );
                Ok(session)
            }
        }
    } else {
        Ok(session)
    }
}
