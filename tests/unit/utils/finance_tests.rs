use ig_client::application::models::account::{Position, PositionDetails, PositionMarket};
use ig_client::application::models::order::Direction;
use ig_client::utils::finance::{calculate_percentage_return, calculate_pnl};
use tracing::info;

#[test]
fn test_calculate_pnl() {
    // Test profit for long position
    let position = create_test_position(Direction::Buy, 100.0, 1.2000, 1.2100, 1.2050);
    let pnl = calculate_pnl(&position);
    info!("Test 1 - Long profit - Actual PNL: {}", pnl.unwrap());
    assert_eq!(pnl.unwrap(), 1.0000000000000009);

    // Test loss for long position
    let position = create_test_position(Direction::Buy, 100.0, 1.2100, 1.1900, 1.1950);
    let pnl = calculate_pnl(&position);
    info!("Test 2 - Long loss - Actual PNL: {}", pnl.unwrap());
    let expected = -2.0000000000000018;
    assert_eq!(pnl.unwrap(), expected);

    // Test profit for short position
    let position = create_test_position(Direction::Sell, 100.0, 1.2100, 1.1900, 1.1950);
    let pnl = calculate_pnl(&position);
    info!("Test 3 - Short profit - Actual PNL: {}", pnl.unwrap());
    let expected = 1.4999999999999902;
    assert_eq!(pnl.unwrap(), expected);

    // Test loss for short position
    let position = create_test_position(Direction::Sell, 100.0, 1.1900, 1.2100, 1.2050);
    let pnl = calculate_pnl(&position);
    info!("Test 4 - Short loss - Actual PNL: {}", pnl.unwrap());
    let expected = -1.5000000000000124;
    assert_eq!(pnl.unwrap(), expected);

    // Test with zero position size
    let position = create_test_position(Direction::Buy, 0.0, 1.2000, 1.2100, 1.2050);
    let pnl = calculate_pnl(&position);
    info!("Test 5 - Zero size - Actual PNL: {}", pnl.unwrap());
    assert_eq!(pnl.unwrap(), 0.0);
}

#[test]
fn test_calculate_percentage_return() {
    // Test profit percentage for long position
    let position = create_test_position(Direction::Buy, 100.0, 1.2000, 1.2100, 1.2050);
    let percentage = calculate_percentage_return(&position);
    info!(
        "Test 1 - Long profit - Actual percentage: {}",
        percentage.unwrap()
    );
    let expected = 0.833333333333334;
    assert_eq!(percentage.unwrap(), expected);

    // Test loss percentage for long position
    let position = create_test_position(Direction::Buy, 100.0, 1.2100, 1.1900, 1.1950);
    let percentage = calculate_percentage_return(&position);
    info!(
        "Test 2 - Long loss - Actual percentage: {}",
        percentage.unwrap()
    );
    let expected = -1.6528925619834725;
    assert_eq!(percentage.unwrap(), expected);

    // Test profit percentage for short position
    let position = create_test_position(Direction::Sell, 100.0, 1.2100, 1.1900, 1.1950);
    let percentage = calculate_percentage_return(&position);
    info!(
        "Test 3 - Short profit - Actual percentage: {}",
        percentage.unwrap()
    );
    let expected = 1.2396694214875952;
    assert_eq!(percentage.unwrap(), expected);

    // Test with zero position size
    let position = create_test_position(Direction::Buy, 0.0, 1.2000, 1.2100, 1.2050);
    let percentage = calculate_percentage_return(&position);
    info!("Test 4 - Zero size - Actual percentage: {:?}", percentage);
    assert_eq!(percentage, None); // Should return None to avoid division by zero

    // Test with zero entry price
    let position = create_test_position(Direction::Buy, 100.0, 0.0, 1.2100, 1.2050);
    let percentage = calculate_percentage_return(&position);
    info!("Test 5 - Zero entry - Actual percentage: {:?}", percentage);
    assert_eq!(percentage, None); // Should return None to avoid division by zero
}

// Helper function to create a test position
fn create_test_position(
    direction: Direction,
    size: f64,
    level: f64,
    bid: f64,
    offer: f64,
) -> Position {
    Position {
        position: PositionDetails {
            contract_size: 1.0,
            created_date: "2023-01-01T00:00:00".to_string(),
            created_date_utc: "2023-01-01T00:00:00Z".to_string(),
            deal_id: "DEAL123".to_string(),
            deal_reference: "REF123".to_string(),
            direction,
            limit_level: None,
            level,
            size,
            stop_level: None,
            trailing_step: None,
            trailing_stop_distance: None,
            controlled_risk: false,
            currency: "USD".to_string(),
            limited_risk_premium: None,
        },
        market: PositionMarket {
            bid: Some(bid),
            delay_time: 0,
            epic: "OP.D.OTCDAX1.021100P.IP".to_string(),
            expiry: "-".to_string(),
            high: Some(0.0),
            instrument_name: "EUR/USD".to_string(),
            instrument_type: "CURRENCIES".to_string(),
            lot_size: 1.0,
            low: Some(0.0),
            market_status: "TRADEABLE".to_string(),
            net_change: 0.0,
            offer: Some(offer),
            percentage_change: 0.0,
            scaling_factor: 1,
            streaming_prices_available: true,
            update_time: "2023-01-01T00:00:00".to_string(),
            update_time_utc: "2023-01-01T00:00:00Z".to_string(),
        },
        pnl: None,
    }
}
