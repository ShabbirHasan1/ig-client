//! examples/login_example.rs
//! cargo run --example login_example

use ig_client::config::Config;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::utils::rate_limiter::RateLimitType;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Simple console logger
    let sub = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(sub).expect("setting default subscriber failed");

    // 1. Load config from env (see Config::new)
    let cfg = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    info!("Loaded config → {}", cfg.rest_api.base_url);
    // 2. Instantiate authenticator
    let auth = IgAuth::new(&cfg);

    // 3. Try login
    match auth.login().await {
        Ok(sess) => {
            info!("✅ Auth ok. Account: {}", sess.account_id);
            info!("CST  = {}", sess.cst);
            info!("X-ST = {}", sess.token);
        }
        Err(e) => {
            error!("Auth failed: {e:?}");
        }
    }
}
