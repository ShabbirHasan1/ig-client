use ig_client::config::Config;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::utils::logger::setup_logger;
use std::error::Error;
use std::sync::Arc;
use chrono::{Duration, Utc};
use tracing::{error, info};
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::application::services::AccountService;
use ig_client::transport::http_client::IgHttpClientImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    setup_logger();

    // Load configuration from environment variables
    let cfg = Config::new();
    info!("Loaded config → {}", cfg.rest_api.base_url);

    // Create authenticator
    let auth = IgAuth::new(&cfg);

    // Login to get initial session
    info!("Logging in...");
    let mut session = match auth.login().await {
        Ok(sess) => {
            info!("✅ Authentication successful. Account: {}", sess.account_id);
            info!("CST  = {}", sess.cst);
            info!("X-ST = {}", sess.token);
            sess
        }
        Err(e) => {
            error!("Authentication failed: {e:?}");
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };

    // Display the current account ID
    info!("Current account ID: {}", session.account_id);
    let account_id = cfg.credentials.account_id.trim().to_string();

    session = if !account_id.is_empty() && session.account_id != account_id {

        let default_account = false;

        // Switch to the specified account
        info!("Switching to account: {}", account_id);
        match auth
            .switch_account(&session, &account_id, Some(default_account))
            .await
        {
            Ok(new_session) => {
                info!("✅ Account switch successful");
                info!("New account ID: {}", new_session.account_id);
                info!("Default account: {}", default_account);
                new_session
            }
            Err(e) => {
                error!("Account switch failed: {e:?}");
                // If the error is due to API rate limit, we show a clearer message
                if let Some(_err_msg) = e.to_string().find("exceeded-api-key-allowance") {
                    error!(
                        "API rate limit exceeded. Please wait a few minutes before trying again."
                    );
                }
                return Err(Box::new(e) as Box<dyn Error>);
            }
        }
    } else {
        info!("Account switch skipped - no account ID provided");
        session       
    };

    let http_client = Arc::new(IgHttpClientImpl::new(Arc::new(cfg.clone())));
    let account_service = AccountServiceImpl::new(Arc::new(cfg.clone()), Arc::clone(&http_client));
    // Calculate date range
    let to = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let from = (Utc::now() - Duration::days(cfg.days_to_look_back))
        .format("%Y-%m-%dT%H:%M:%S")
        .to_string();

    info!("Fetching transactions from {} to {}", from, to);

    // Fetch all transactions using the updated method
    let _all_transactions = match account_service
        .get_transactions(&session, &from, &to)
        .await
    {
        Ok(transactions) => transactions,
        Err(e) => {
            error!("Failed to fetch transactions: {e:?}");
            return Err(Box::new(e) as Box<dyn Error>);
        }
    };
    
    Ok(())
}
