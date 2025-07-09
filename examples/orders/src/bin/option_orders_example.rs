use ig_client::application::models::order::{
    ClosePositionRequest, CreateOrderRequest, Direction, Status,
};
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::application::services::order_service::OrderServiceImpl;
use ig_client::application::services::{MarketService, OrderService};
use ig_client::utils::rate_limiter::RateLimitType;
use ig_client::{
    config::Config, session::auth::IgAuth, session::interface::IgAuthenticator,
    transport::http_client::IgHttpClientImpl, utils::logger::setup_logger,
};
use nanoid::nanoid;
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::TradingAccount,
        0.01,
    ));
    let config_no_trade = Arc::new(Config::with_rate_limit_type(
        RateLimitType::OnePerSecond,
        0.5,
    ));

    info!("Configuration loaded");

    // Create HTTP client
    let client = Arc::new(IgHttpClientImpl::new(Arc::clone(&config)));
    info!("HTTP client created");

    // Create authenticator
    let authenticator = IgAuth::new(&config);
    info!("Authenticator created");

    // Login to IG
    info!("Logging in to IG...");
    let session = authenticator.login().await?;
    info!("Session started successfully");

    let epic = "DO.D.OTCDDAX.68.IP"; // Example epic for testing
    let expiry = Some(
        chrono::Local::now()
            .format("%d-%b-%y")
            .to_string()
            .to_uppercase(),
    );
    let size = 1.25; // Size of the order
    let currency_code = Some("EUR".to_string()); // Example currency code
    let deal_reference = Some(nanoid!(30, &nanoid::alphabet::SAFE));
    info!("{:?}", deal_reference);
    let create_order = CreateOrderRequest::buy_option_to_market(
        &epic.to_string(),
        &size,
        &expiry.clone(),
        &deal_reference,
        &currency_code.clone(),
    );

    // Create a market service
    // let market_service = MarketServiceImpl::new(config_no_trade, client.clone());
    // Create order service
    let order_service = OrderServiceImpl::new(config, client);
    // Create the position
    let create_result = order_service.create_order(&session, &create_order).await;
    let deal_id: Option<String> = match create_result {
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

            // Ensure we have a deal ID
            match (
                confirmation.status == Status::Rejected,
                confirmation.deal_id,
            ) {
                (true, _) => {
                    error!("Order was rejected, cannot continue");
                    None
                }
                (false, Some(id)) => Some(id),
                (false, None) => {
                    error!("No deal ID received, cannot continue");
                    None
                }
            }
        }
        Err(e) => {
            error!("Failed to create position: {:?}", e);
            None
        }
    };

    if let Some(deal_id) = &deal_id {
        info!("Deal ID obtained: {}", deal_id);
    } else {
        error!("No valid deal ID obtained, exiting");
        return Ok(());
    }

    // sleep for a while to simulate some processing time
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    info!("Closing position with deal ID: {:?}", deal_id);
    let close_request = ClosePositionRequest::close_option_to_market_by_id(
        deal_id.unwrap(),
        Direction::Sell, // Assuming we are closing a buy position
        size,
    );
    // let close_request = ClosePositionRequest::close_option_to_market_by_epic(
    //     epic.to_string(),
    //     expiry.clone().unwrap(),
    //     Direction::Sell, // Assuming we are closing a buy position
    //     size,
    // );
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

            match close_confirmation.status {
                Status::Rejected => {
                    error!("Close order was rejected: {:?}", close_confirmation.reason);
                }
                Status::Open => {
                    error!(
                        "Wrong side, we opened a new position instead of closing: {:?}",
                        close_confirmation.reason
                    );
                }
                Status::Closed => {
                    info!(
                        "Position closed successfully with deal ID: {:?}",
                        close_confirmation.deal_id
                    );
                }
                _ => {
                    warn!("Undefined situation: {:?}", close_confirmation.status);
                }
            }
        }
        Err(e) => {
            info!("Failed to close position: {:?}", e);
        }
    }

    Ok(())
}
