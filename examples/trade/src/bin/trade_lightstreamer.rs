use ig_client::application::services::Listener;
use ig_client::config::Config;
use ig_client::error::AppError;
use ig_client::presentation::TradeData;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::utils::logger::setup_logger;
use lightstreamer_rs::client::{LightstreamerClient, Transport};
use lightstreamer_rs::subscription::{Snapshot, Subscription, SubscriptionMode};
use lightstreamer_rs::utils::setup_signal_hook;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tracing::{error, info, warn};

const MAX_CONNECTION_ATTEMPTS: u64 = 3;

fn callback(update: &TradeData) -> Result<(), AppError> {
    let item = serde_json::to_string_pretty(&update)?;
    info!("TradeData: {}", item);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let config = Arc::new(Config::new());
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    info!("Logging in to IG...");
    let session = match authenticator.login().await {
        Ok(session) => {
            info!("Session started successfully");
            info!("Account ID: {}", session.account_id);
            info!("CST Token length: {}", session.cst.len());
            info!("Security Token length: {}", session.token.len());
            session
        }
        Err(e) => {
            error!("Failed to log in: {}", e);
            error!("Please check your credentials in the .env file");
            error!("Make sure your IG_USERNAME, IG_PASSWORD, and IG_API_KEY are correct");
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };
    info!("Login successful");

    // Determine if we're in demo environment based on the WebSocket URL in config
    let ws_url = &config.websocket.url;
    let is_demo = ws_url.contains("demo");

    // Determine the server address based on environment
    let server_address = if is_demo {
        "https://demo-apd.marketdatasystems.com/lightstreamer"
    } else {
        "https://apd.marketdatasystems.com/lightstreamer"
    };

    info!("Using Lightstreamer server: {}", server_address);
    info!("Using account ID: {}", session.account_id.trim());

    // Format the password as required by IG's Lightstreamer authentication
    let cst = session.cst.trim();
    let token = session.token.trim();
    let password = format!("CST-{}|XST-{}", cst, token);

    info!("Using CST token of length: {}", cst.len());
    info!("Using XST token of length: {}", token.len());

    // Create a subscription for a market
    let epic = format!("TRADE:{}", session.account_id);
    let mut subscription = Subscription::new(
        SubscriptionMode::Distinct,
        Some(vec![epic]),
        Some(vec![
            "CONFIRMS".to_string(),
            "OPU".to_string(),
            "WOU".to_string(),
        ]),
    )?;

    let listener = Listener::new(callback);
    subscription.set_data_adapter(None)?;
    subscription.set_requested_snapshot(Some(Snapshot::Yes))?;
    subscription.add_listener(Box::new(listener));

    // Create a new Lightstreamer client instance and wrap it in an Arc<Mutex<>> so it can be shared across threads.
    let client = Arc::new(Mutex::new(LightstreamerClient::new(
        Some(server_address),
        None,
        Some(session.account_id.trim()),
        Some(&password),
    )?));

    //
    // Add the subscription to the client.
    //
    {
        let mut client = client.lock().await;
        LightstreamerClient::subscribe(client.subscription_sender.clone(), subscription).await;
        client
            .connection_options
            .set_forced_transport(Some(Transport::WsStreaming));
    }

    // Create a new Notify instance to send a shutdown signal to the signal handler thread.
    let shutdown_signal = Arc::new(Notify::new());
    // Spawn a new thread to handle SIGINT and SIGTERM process signals.
    setup_signal_hook(Arc::clone(&shutdown_signal)).await;

    //
    // Infinite loop that will indefinitely retry failed connections unless
    // a SIGTERM or SIGINT signal is received.
    //
    let mut retry_interval_milis: u64 = 0;
    let mut retry_counter: u64 = 0;
    while retry_counter < MAX_CONNECTION_ATTEMPTS {
        let mut client = client.lock().await;
        match client.connect_direct(Arc::clone(&shutdown_signal)).await {
            Ok(_) => {
                client.disconnect().await;
                break;
            }
            Err(e) => {
                error!("Failed to connect: {:?}", e);
                tokio::time::sleep(std::time::Duration::from_millis(retry_interval_milis)).await;
                retry_interval_milis = (retry_interval_milis + (200 * retry_counter)).min(5000);
                retry_counter += 1;
                warn!(
                    "Retrying connection in {} seconds...",
                    format!("{:.2}", retry_interval_milis as f64 / 1000.0)
                );
            }
        }
    }

    if retry_counter == MAX_CONNECTION_ATTEMPTS {
        error!(
            "Failed to connect after {} retries. Exiting...",
            retry_counter
        );
    } else {
        info!("Exiting orderly from Lightstreamer client...");
    }

    // Exit using std::process::exit() to avoid waiting for existing tokio tasks to complete.
    std::process::exit(0);
}
