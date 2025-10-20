// Integration tests for position endpoints

use crate::common;
use ig_client::application::services::AccountService;
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::utils::logger::setup_logger;
use tokio::runtime::Runtime;
use tracing::info;

#[test]
#[ignore]
fn test_get_positions() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create account service
    let account_service = AccountServiceImpl::new(config.clone(), client.clone());

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get positions
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per account)
        ig_client::utils::rate_limiter::account_non_trading_limiter()
            .wait()
            .await;
        info!("Getting open positions");

        let result = account_service
            .get_positions(&session)
            .await
            .expect("Failed to get positions");

        // Print the positions
        info!("Retrieved {} open positions", result.positions.len());

        // Print details of each position
        if result.positions.is_empty() {
            info!("No open positions found");
        } else {
            for (i, position) in result.positions.iter().enumerate() {
                info!(
                    "{}.  {} ({})",
                    i + 1,
                    position.market.instrument_name,
                    position.market.epic
                );
                info!(
                    "   Direction: {}, Size: {}",
                    position.position.direction, position.position.size
                );
                let current_level_display = match position.position.direction {
                    ig_client::application::models::order::Direction::Buy => position
                        .market
                        .bid
                        .map(|b| format!("{}", b))
                        .unwrap_or_else(|| "N/A".to_string()),
                    ig_client::application::models::order::Direction::Sell => position
                        .market
                        .offer
                        .map(|o| format!("{}", o))
                        .unwrap_or_else(|| "N/A".to_string()),
                };

                info!(
                    "   Open Level: {}, Current Level: {}",
                    position.position.level, current_level_display
                );

                // Calculate profit/loss if possible
                let open_level = position.position.level;
                let current_level_opt = match position.position.direction {
                    ig_client::application::models::order::Direction::Buy => position.market.bid,
                    ig_client::application::models::order::Direction::Sell => position.market.offer,
                };

                if let Some(current_level) = current_level_opt {
                    let direction_multiplier = match position.position.direction {
                        ig_client::application::models::order::Direction::Buy => 1.0,
                        ig_client::application::models::order::Direction::Sell => -1.0,
                    };

                    let points = (current_level - open_level) * direction_multiplier;
                    let pnl = points * position.position.size;

                    info!("   Points: {:.2}, P/L: {:.2}", points, pnl);
                } else {
                    info!("   P/L: N/A (no current price available)");
                }
            }
        }
    });
}

#[test]
#[ignore]
fn test_get_working_orders() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create account service
    let account_service = AccountServiceImpl::new(config.clone(), client.clone());

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get working orders
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per account)
        ig_client::utils::rate_limiter::account_non_trading_limiter()
            .wait()
            .await;
        info!("Getting working orders");

        let result = account_service
            .get_working_orders(&session)
            .await
            .expect("Failed to get working orders");

        // Print the working orders
        info!("Retrieved {} working orders", result.working_orders.len());

        // Print details of each working order
        if result.working_orders.is_empty() {
            info!("No working orders found");
        } else {
            for (i, order) in result.working_orders.iter().enumerate() {
                info!(
                    "{}.  {} ({})",
                    i + 1,
                    order.market_data.instrument_name,
                    order.market_data.epic
                );
                info!(
                    "   Direction: {}, Size: {}",
                    order.working_order_data.direction, order.working_order_data.order_size
                );
                info!(
                    "   Order Type: {:?}, Level: {}",
                    order.working_order_data.order_type, order.working_order_data.order_level
                );
                info!(
                    "   Created: {:?}, Good Till: {:?}",
                    order.working_order_data.created_date, order.working_order_data.good_till_date
                );
            }
        }
    });
}
