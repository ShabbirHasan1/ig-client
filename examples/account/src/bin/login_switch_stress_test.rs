use ig_client::config::Config;
use ig_client::session::auth::IgAuth;
use ig_client::session::interface::IgAuthenticator;
use ig_client::utils::logger::setup_logger;
use std::error::Error;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    setup_logger();

    // Load configuration from environment variables
    let cfg = Config::new();
    info!("Loaded config → {}", cfg.rest_api.base_url);
    info!("Target account ID: {}", cfg.credentials.account_id);

    // Create authenticator
    let auth = IgAuth::new(&cfg);

    let account_id = cfg.credentials.account_id.trim().to_string();

    if account_id.is_empty() {
        error!("No account ID configured. Please set IG_ACCOUNT_ID environment variable.");
        return Err("Missing account ID configuration".into());
    }

    info!("Starting login_and_switch_account stress test...");
    info!("Will attempt login and switch every second. Press Ctrl+C to stop.");

    let mut attempt_count = 0;
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut consecutive_failures = 0;
    let max_consecutive_failures = 5;

    let start_time = Instant::now();

    loop {
        attempt_count += 1;
        let attempt_start = Instant::now();

        info!("\n=== Attempt #{} ===", attempt_count);

        match auth
            .login_and_switch_account(&account_id, Some(false))
            .await
        {
            Ok(session) => {
                let duration = attempt_start.elapsed();
                success_count += 1;
                consecutive_failures = 0;

                info!("✅ SUCCESS - Attempt #{}", attempt_count);
                info!("   Account ID: {}", session.account_id);
                info!("   CST length: {}", session.cst.len());
                info!("   Token length: {}", session.token.len());
                info!("   Duration: {:?}", duration);

                // Log statistics every 10 successful attempts
                if success_count % 10 == 0 {
                    let total_time = start_time.elapsed();
                    let success_rate = (success_count as f64 / attempt_count as f64) * 100.0;
                    info!("\n📊 STATISTICS (after {} attempts):", attempt_count);
                    info!("   Successes: {} ({:.1}%)", success_count, success_rate);
                    info!(
                        "   Failures: {} ({:.1}%)",
                        failure_count,
                        100.0 - success_rate
                    );
                    info!("   Total time: {:?}", total_time);
                    info!(
                        "   Average time per attempt: {:?}",
                        total_time / attempt_count
                    );
                }
            }
            Err(e) => {
                let duration = attempt_start.elapsed();
                failure_count += 1;
                consecutive_failures += 1;

                error!("❌ FAILURE - Attempt #{}", attempt_count);
                error!("   Error: {:?}", e);
                error!("   Duration: {:?}", duration);
                error!("   Consecutive failures: {}", consecutive_failures);

                // Check for specific error patterns
                let error_str = format!("{:?}", e);
                if error_str.contains("exceeded-api-key-allowance") {
                    warn!("   🚨 Rate limit exceeded detected");
                } else if error_str.contains("account-token-invalid") {
                    warn!("   🚨 Invalid account token detected");
                } else if error_str.contains("Unauthorized") {
                    warn!("   🚨 Unauthorized error detected");
                } else if error_str.contains("timeout") {
                    warn!("   🚨 Timeout error detected");
                } else {
                    warn!("   🚨 Unknown error pattern");
                }

                // If too many consecutive failures, add a longer delay
                if consecutive_failures >= max_consecutive_failures {
                    warn!("   ⏸️  Too many consecutive failures, adding 10 second delay...");
                    sleep(Duration::from_secs(10)).await;
                    consecutive_failures = 0; // Reset counter after delay
                }
            }
        }

        // Log periodic statistics
        if attempt_count % 50 == 0 {
            let total_time = start_time.elapsed();
            let success_rate = (success_count as f64 / attempt_count as f64) * 100.0;
            info!("\n📈 PERIODIC REPORT (after {} attempts):", attempt_count);
            info!(
                "   Success rate: {:.1}% ({}/{})",
                success_rate, success_count, attempt_count
            );
            info!(
                "   Failure rate: {:.1}% ({})",
                100.0 - success_rate,
                failure_count
            );
            info!("   Total runtime: {:?}", total_time);
            info!(
                "   Average time per attempt: {:?}",
                total_time / attempt_count
            );

            // If success rate is very low, warn the user
            if success_rate < 50.0 && attempt_count >= 20 {
                warn!("   ⚠️  Low success rate detected! Check API credentials and network.");
            }
        }

        // Wait 1 second before next attempt
        sleep(Duration::from_secs(1)).await;
    }
}
