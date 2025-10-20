// Integration tests for order service endpoints

use crate::common;
use ig_client::prelude::*;
use ig_client::presentation::order::{ClosePositionRequest, CreateOrderRequest, Direction, TimeInForce, UpdatePositionRequest};
use tokio::runtime::Runtime;
use tracing::{info, warn};

#[test]
#[ignore]
fn test_create_and_close_position() {
    setup_logger();
    // Create client
    let client = common::create_test_client();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test create and close position
    rt.block_on(async {
        info!("Creating a test position");

        // Get current market price to set a reasonable limit price
        use ig_client::application::services::MarketService;
        use ig_client::application::services::market_service::MarketServiceImpl;

        let market_service = MarketServiceImpl::new(
            common::create_test_config(),
            common::create_test_client(common::create_test_config()),
        );

        // Test both an open and a closed market epic
        let open_epic = "OP.D.OTCDAX1.021100P.IP"; // Epic for an open market
        let closed_epic = "DO.D.OTCDDAX.71.IP";    // Epic for a closed market that we know returns MARKET_CLOSED_WITH_EDITS

        // First try with the open market
        info!("Testing with open market epic: {}", open_epic);
        let market_details_result = market_service
            .get_market_details(&session, open_epic)
            .await;

        // If the open market fails, try the closed market
        let (epic, market_details) = match market_details_result {
            Ok(details) => {
                info!("Successfully got details for open market");
                (open_epic, details)
            },
            Err(e) => {
                info!("Open market failed: {:?}, trying closed market", e);
                // Now try with the closed market
                info!("Testing with closed market epic: {}", closed_epic);
                match market_service.get_market_details(&session, closed_epic).await {
                    Ok(details) => {
                        info!("Successfully got details for closed market");
                        (closed_epic, details)
                    },
                    Err(e) => {
                        panic!("Both open and closed market requests failed: {e:?}");
                    }
                }
            }
        };

        // Get current price and set limit price slightly higher for a buy order
        let current_price = market_details.snapshot.offer.unwrap_or(100.0);
        let limit_price = current_price + 5.0; // Set limit price 5 points above current price

        info!(
            "Current price: {}, setting limit price to: {}",
            current_price, limit_price
        );

        // Create a small test position using a limit order
        let mut create_order = CreateOrderRequest::limit(
            epic.to_string(),
            Direction::Buy,
            0.2, // Very small size to minimize risk
            limit_price,
            None,
            None,
        )
        .with_reference(format!("test_{}", chrono::Utc::now().timestamp()));

        // Set required fields
        create_order.expiry = Some("JUL-25".to_string()); // Use actual expiry date for options
        create_order.guaranteed_stop = false; // Specify whether to use a guaranteed stop
        create_order.currency_code = "EUR".to_string(); // Set the currency code for the order
        create_order.time_in_force = TimeInForce::FillOrKill; // Use fill or kill

        // Create the position
        let create_result = order_service.create_order(&session, &create_order).await;

        match create_result {
            Ok(response) => {
                info!(
                    "Position created with deal reference: {}",
                    response.deal_reference
                );

                // Get the order confirmation to obtain the deal ID
                let confirmation = order_service
                    .get_order_confirmation(&session, &response.deal_reference)
                    .await
                    .expect("Failed to get order confirmation");

                info!("Order confirmation received:");
                info!("  Deal ID: {:?}", confirmation.deal_id);
                info!("  Status: {:?}", confirmation.status);
                info!("  Reason: {:?}", confirmation.reason);

                // Check if this is a closed market response
                if confirmation.reason.as_deref() == Some("MARKET_CLOSED_WITH_EDITS") {
                    info!("✅ Successfully verified serialization of MARKET_CLOSED_WITH_EDITS response");
                    info!("✅ Status field is correctly handled as: {:?}", confirmation.status);

                    // Verify that all fields are correctly deserialized
                    assert!(confirmation.deal_id.is_some(), "deal_id should be present");
                    // For closed markets, status could be null in the JSON but should be deserialized as Rejected
                    assert_eq!(confirmation.status, ig_client::application::models::order::Status::Rejected,
                              "Status should be Rejected for closed markets");
                    assert_eq!(confirmation.reason, Some("MARKET_CLOSED_WITH_EDITS".to_string()),
                              "Reason should be MARKET_CLOSED_WITH_EDITS");
                }

                if confirmation.status == ig_client::application::models::order::Status::Rejected {
                    info!("Order was rejected: {:?}", confirmation.reason);
                    return;
                }

                // Ensure we have a deal ID
                let deal_id = match confirmation.deal_id {
                    Some(id) => id,
                    None => {
                        info!("No deal ID received, cannot continue");
                        return;
                    }
                };

                // Wait a moment to ensure the position is fully processed
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                // Get current market price to set a reasonable limit price for closing
                let market_details = market_service
                    .get_market_details(&session, epic)
                    .await
                    .expect("Failed to get market details");

                // Get current price and set limit price based on direction
                let close_price = match create_order.direction {
                    Direction::Buy => market_details.snapshot.bid.unwrap_or(100.0) - 5.0, // Set lower for selling
                    Direction::Sell => market_details.snapshot.offer.unwrap_or(100.0) + 5.0, // Set higher for buying
                };

                info!(
                    "Current price for closing: {}, setting limit price to: {}",
                    match create_order.direction {
                        Direction::Buy => market_details.snapshot.bid.unwrap_or(100.0),
                        Direction::Sell => market_details.snapshot.offer.unwrap_or(100.0),
                    },
                    close_price
                );

                // Close the position using a limit order
                let close_request = ClosePositionRequest::limit(
                    deal_id.clone(),
                    match create_order.direction {
                        Direction::Buy => Direction::Sell,
                        Direction::Sell => Direction::Buy,
                    },
                    create_order.size,
                    close_price,
                );

                info!("Closing position with deal ID: {}", deal_id);

                let close_result = order_service.close_position(&session, &close_request).await;

                match close_result {
                    Ok(close_response) => {
                        info!(
                            "Position closed with deal reference: {}",
                            close_response.deal_reference
                        );

                        // Get the close confirmation
                        let close_confirmation = order_service
                            .get_order_confirmation(&session, &close_response.deal_reference)
                            .await
                            .expect("Failed to get close confirmation");

                        info!("Close confirmation received:");
                        info!("  Deal ID: {:?}", close_confirmation.deal_id);
                        info!("  Status: {:?}", close_confirmation.status);
                        info!("  Reason: {:?}", close_confirmation.reason);
                    }
                    Err(e) => {
                        info!("Failed to close position: {:?}", e);
                    }
                }
            }
            Err(e) => {
                info!("Failed to create position: {:?}", e);
            }
        }
    });
}

#[test]
#[ignore]
fn test_closed_market_serialization() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create order service
    let order_service = OrderServiceImpl::new(config.clone(), client.clone());

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test creating an order in a closed market
    rt.block_on(async {
        // Wait to respect the rate limit
        ig_client::utils::rate_limiter::account_trading_limiter()
            .wait()
            .await;

        info!("Testing order creation with a known closed market");

        // Use a market that we know is closed
        let closed_epic = "DO.D.OTCDDAX.71.IP";

        // Create a test order for the closed market
        let mut create_order = CreateOrderRequest::limit(
            closed_epic.to_string(),
            Direction::Buy,
            0.2,   // Small size
            100.0, // Arbitrary price
            None,
            None,
        )
        .with_reference(format!("test_closed_{}", chrono::Utc::now().timestamp()));

        // Set required fields
        create_order.expiry = Some("JUL-25".to_string());
        create_order.guaranteed_stop = false;
        create_order.currency_code = "EUR".to_string();
        create_order.time_in_force = ig_client::application::models::order::TimeInForce::FillOrKill;

        // Attempt to create the position (should be rejected due to closed market)
        let create_result = order_service.create_order(&session, &create_order).await;

        match create_result {
            Ok(response) => {
                info!(
                    "Got deal reference for closed market: {}",
                    response.deal_reference
                );

                // Get the order confirmation to verify serialization
                let confirmation = order_service
                    .get_order_confirmation(&session, &response.deal_reference)
                    .await;

                match confirmation {
                    Ok(conf) => {
                        info!("✅ Successfully parsed confirmation for closed market");
                        info!("  Deal ID: {:?}", conf.deal_id);
                        info!("  Status: {:?}", conf.status);
                        info!("  Reason: {:?}", conf.reason);

                        // Verify the closed market response fields
                        assert!(conf.deal_id.is_some(), "deal_id should be present");
                        assert_eq!(
                            conf.status,
                            ig_client::application::models::order::Status::Rejected,
                            "Status should be Rejected for closed markets"
                        );
                        assert_eq!(
                            conf.reason,
                            Some("MARKET_CLOSED_WITH_EDITS".to_string()),
                            "Reason should be MARKET_CLOSED_WITH_EDITS"
                        );

                        info!("✅ Closed market serialization test passed");
                    }
                    Err(e) => {
                        panic!("Failed to get confirmation for closed market: {e:?}");
                    }
                }
            }
            Err(e) => {
                panic!("Failed to create order for closed market test: {e:?}");
            }
        }
    });
}

#[test]
#[ignore]
fn test_update_position() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create order service and account service
    let order_service = OrderServiceImpl::new(config.clone(), client.clone());
    let account_service =
        ig_client::application::services::account_service::AccountServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test update position
    rt.block_on(async {
        // Wait to respect the rate limit (trading requests per account)
        ig_client::utils::rate_limiter::account_trading_limiter()
            .wait()
            .await;
        // First get all positions to find one to update
        use ig_client::application::services::AccountService;

        let positions = account_service
            .get_positions(&session)
            .await
            .expect("Failed to get positions");

        if positions.positions.is_empty() {
            info!("No open positions found, creating a test position");

            // Get current market price to set a reasonable limit price
            use ig_client::application::services::MarketService;
            use ig_client::application::services::market_service::MarketServiceImpl;

            let market_service = MarketServiceImpl::new(
                common::create_test_config(),
                common::create_test_client(common::create_test_config()),
            );

            // Test both an open and a closed market epic
            let open_epic = "OP.D.OTCDAX1.021100P.IP"; // Epic for an open market
            let closed_epic = "DO.D.OTCDDAX.71.IP"; // Epic for a closed market that we know returns MARKET_CLOSED_WITH_EDITS

            // First try with the open market
            info!("Testing with open market epic: {}", open_epic);
            let market_details_result =
                market_service.get_market_details(&session, open_epic).await;

            // If the open market fails, try the closed market
            let (epic, market_details) = match market_details_result {
                Ok(details) => {
                    info!("Successfully got details for open market");
                    (open_epic, details)
                }
                Err(e) => {
                    info!("Open market failed: {:?}, trying closed market", e);
                    // Now try with the closed market
                    info!("Testing with closed market epic: {}", closed_epic);
                    match market_service
                        .get_market_details(&session, closed_epic)
                        .await
                    {
                        Ok(details) => {
                            info!("Successfully got details for closed market");
                            (closed_epic, details)
                        }
                        Err(e) => {
                            panic!("Both open and closed market requests failed: {e:?}");
                        }
                    }
                }
            };

            // Get current price and set limit price slightly higher for a buy order
            let current_price = market_details.snapshot.offer.unwrap_or(100.0);
            let limit_price = current_price + 5.0; // Set limit price 5 points above current price

            info!(
                "Current price: {}, setting limit price to: {}",
                current_price, limit_price
            );

            // Create a small test position using a limit order
            let mut create_order = CreateOrderRequest::limit(
                epic.to_string(),
                Direction::Buy,
                0.1, // Very small size to minimize risk
                limit_price,
                None,
                None,
            )
            .with_reference(format!("test_{}", chrono::Utc::now().timestamp()));

            // Set required fields
            create_order.expiry = Some("JUL-25".to_string()); // Use actual expiry date for options
            create_order.guaranteed_stop = false; // Specify whether to use a guaranteed stop
            create_order.currency_code = "EUR".to_string(); // Set the currency code for the order
            create_order.time_in_force =
                ig_client::application::models::order::TimeInForce::FillOrKill; // Use fill or kill

            // Create the position
            let create_result = order_service.create_order(&session, &create_order).await;

            match create_result {
                Ok(response) => {
                    info!(
                        "Position created with deal reference: {}",
                        response.deal_reference
                    );

                    // Get the order confirmation to obtain the deal ID
                    let confirmation = order_service
                        .get_order_confirmation(&session, &response.deal_reference)
                        .await
                        .expect("Failed to get order confirmation");

                    if confirmation.status
                        == ig_client::application::models::order::Status::Rejected
                    {
                        info!("Order was rejected: {:?}", confirmation.reason);
                        return;
                    }

                    // Ensure we have a deal ID
                    let deal_id = match confirmation.deal_id {
                        Some(id) => id,
                        None => {
                            info!("No deal ID received, cannot continue");
                            return;
                        }
                    };

                    // Wait a moment to ensure the position is fully processed
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                    // Get current price to set reasonable stop and limit levels
                    let market_service =
                        ig_client::application::services::market_service::MarketServiceImpl::new(
                            common::create_test_config(),
                            common::create_test_client(common::create_test_config()),
                        );

                    use ig_client::application::services::MarketService;

                    // Use the same market we're creating the position for
                    let market_details = market_service
                        .get_market_details(&session, epic)
                        .await
                        .expect("Failed to get market details");

                    // Unwrap the price values with defaults if they're None
                    let current_price = match create_order.direction {
                        Direction::Buy => market_details.snapshot.offer.unwrap_or(100.0),
                        Direction::Sell => market_details.snapshot.bid.unwrap_or(100.0),
                    };

                    info!("Current price: {}", current_price);

                    // Set stop 20 points below and limit 20 points above for a buy
                    let (stop_level, limit_level) = match create_order.direction {
                        Direction::Buy => (current_price - 20.0, current_price + 20.0),
                        Direction::Sell => (current_price + 20.0, current_price - 20.0),
                    };

                    let update_request = UpdatePositionRequest {
                        stop_level: Some(stop_level),
                        limit_level: Some(limit_level),
                        trailing_stop: Some(false),
                        trailing_stop_distance: None,
                    };

                    info!("Updating position with deal ID: {}", deal_id);
                    info!("  Setting stop level: {}", stop_level);
                    info!("  Setting limit level: {}", limit_level);

                    let update_result = order_service
                        .update_position(&session, &deal_id, &update_request)
                        .await;

                    match update_result {
                        Ok(_) => {
                            info!("Position updated successfully");

                            // Get current market price to set a reasonable limit price for closing
                            let market_details = market_service
                                .get_market_details(&session, epic)
                                .await
                                .expect("Failed to get market details");

                            // Get current price and set limit price based on direction
                            let close_price = match create_order.direction {
                                Direction::Buy => {
                                    market_details.snapshot.bid.unwrap_or(100.0) - 5.0
                                } // Set lower for selling
                                Direction::Sell => {
                                    market_details.snapshot.offer.unwrap_or(100.0) + 5.0
                                } // Set higher for buying
                            };

                            info!(
                                "Current price for closing: {}, setting limit price to: {}",
                                match create_order.direction {
                                    Direction::Buy => market_details.snapshot.bid.unwrap_or(100.0),
                                    Direction::Sell =>
                                        market_details.snapshot.offer.unwrap_or(100.0),
                                },
                                close_price
                            );

                            // Close the position using a limit order
                            let close_request = ClosePositionRequest::limit(
                                deal_id.clone(),
                                match create_order.direction {
                                    Direction::Buy => Direction::Sell,
                                    Direction::Sell => Direction::Buy,
                                },
                                create_order.size,
                                close_price,
                            );

                            info!("Closing position with deal ID: {}", deal_id);

                            let close_result =
                                order_service.close_position(&session, &close_request).await;

                            match close_result {
                                Ok(_) => info!("Position closed successfully"),
                                Err(e) => info!("Failed to close position: {:?}", e),
                            }
                        }
                        Err(e) => {
                            info!("Failed to update position: {:?}", e);

                            // Get current market price to set a reasonable limit price for closing
                            let market_details = market_service
                                .get_market_details(&session, epic)
                                .await
                                .expect("Failed to get market details");

                            // Get current price and set limit price based on direction
                            let close_price = match create_order.direction {
                                Direction::Buy => {
                                    market_details.snapshot.bid.unwrap_or(100.0) - 5.0
                                } // Set lower for selling
                                Direction::Sell => {
                                    market_details.snapshot.offer.unwrap_or(100.0) + 5.0
                                } // Set higher for buying
                            };

                            info!(
                                "Current price for closing: {}, setting limit price to: {}",
                                match create_order.direction {
                                    Direction::Buy => market_details.snapshot.bid.unwrap_or(100.0),
                                    Direction::Sell =>
                                        market_details.snapshot.offer.unwrap_or(100.0),
                                },
                                close_price
                            );

                            // Try to close the position anyway to clean up
                            let close_request = ClosePositionRequest::limit(
                                deal_id.clone(),
                                match create_order.direction {
                                    Direction::Buy => Direction::Sell,
                                    Direction::Sell => Direction::Buy,
                                },
                                create_order.size,
                                close_price,
                            );

                            info!("Closing position with deal ID: {}", deal_id);

                            let _ = order_service.close_position(&session, &close_request).await;
                        }
                    }
                }
                Err(e) => {
                    info!("Failed to create position: {:?}", e);
                }
            }

            return;
        }

        // Use the first position's deal ID
        let position = &positions.positions[0];
        let deal_id = &position.position.deal_id;
        info!("Updating position with deal ID: {}", deal_id);

        // Get current price
        let current_price_opt = match position.position.direction {
            Direction::Buy => position.market.offer,
            Direction::Sell => position.market.bid,
        };

        // Only proceed if current price is available
        if let Some(current_price) = current_price_opt {
            // Set stop 20 points away from current price
            let (stop_level, limit_level) = match position.position.direction {
                Direction::Buy => (current_price - 20.0, current_price + 20.0),
                Direction::Sell => (current_price + 20.0, current_price - 20.0),
            };

            let update_request = UpdatePositionRequest {
                stop_level: Some(stop_level),
                limit_level: Some(limit_level),
                trailing_stop: Some(false),
                trailing_stop_distance: None,
            };

            info!("  Setting stop level: {}", stop_level);
            info!("  Setting limit level: {}", limit_level);

            let result = order_service
                .update_position(&session, deal_id, &update_request)
                .await;

            match result {
                Ok(_) => info!("Position updated successfully"),
                Err(e) => info!("Failed to update position: {:?}", e),
            }
        } else {
            warn!("  No current price available for position, skipping update");
        }
    });
}
