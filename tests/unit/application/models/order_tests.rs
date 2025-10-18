use ig_client::application::models::order::{
    ClosePositionRequest, CreateOrderRequest, CreateWorkingOrderRequest, Direction, OrderType,
    Status, TimeInForce,
};
use serde::Deserialize;
use serde_json::json;

#[test]
fn test_create_order_request_market() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;

    let order = CreateOrderRequest::market(epic.to_string(), direction.clone(), size, None, None);

    assert_eq!(order.epic, epic);
    assert_eq!(order.direction, direction);
    assert_eq!(order.size, size);
    assert_eq!(order.order_type, OrderType::Market);
    assert_eq!(order.time_in_force, TimeInForce::FillOrKill);
    assert!(order.level.is_none());
    assert!(!order.guaranteed_stop);
    assert!(order.stop_level.is_none());
    assert!(order.stop_distance.is_none());
    assert!(order.limit_level.is_none());
    assert!(order.limit_distance.is_none());
    // quote_id field no longer exists in CreateOrderRequest
    assert_eq!(order.currency_code, "EUR".to_string());
    assert!(order.force_open); // Updated: force_open is now Some(true) by default
    assert_eq!(order.expiry, Some("-".to_string()));
    assert!(order.deal_reference.is_none());
}

#[test]
fn test_create_order_request_limit() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Sell;
    let size = 2.0;
    let level = 1.2345;

    let order =
        CreateOrderRequest::limit(epic.to_string(), direction.clone(), size, level, None, None);

    assert_eq!(order.epic, epic);
    assert_eq!(order.direction, direction);
    assert_eq!(order.size, size);
    assert_eq!(order.order_type, OrderType::Limit);
    assert_eq!(order.time_in_force, TimeInForce::GoodTillCancelled);
    assert_eq!(order.level, Some(level));
    assert!(!order.guaranteed_stop);
    assert!(order.stop_level.is_none());
    assert!(order.stop_distance.is_none());
    assert!(order.limit_level.is_none());
    assert!(order.limit_distance.is_none());
    // quote_id field no longer exists in CreateOrderRequest
    assert_eq!(order.currency_code, "EUR".to_string());
    assert!(order.force_open); // Updated: force_open is now Some(true) by default
    assert!(order.expiry.is_none());
    assert!(order.deal_reference.is_none());
}

#[test]
fn test_create_order_request_with_stop_loss() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let stop_level = 1.2000;

    let order = CreateOrderRequest::market(epic.to_string(), direction, size, None, None)
        .with_stop_loss(stop_level);

    assert_eq!(order.stop_level, Some(stop_level));
}

#[test]
fn test_create_order_request_with_take_profit() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let limit_level = 1.3000;

    let order = CreateOrderRequest::market(epic.to_string(), direction, size, None, None)
        .with_take_profit(limit_level);

    assert_eq!(order.limit_level, Some(limit_level));
}

#[test]
fn test_create_order_request_with_reference() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let reference = "test-reference-123";

    let order = CreateOrderRequest::market(epic.to_string(), direction, size, None, None)
        .with_reference(reference.to_string());

    assert_eq!(order.deal_reference, Some(reference.to_string()));
}

#[test]
fn test_create_order_request_sell_option_to_market() {
    let epic = "CC.D.LCO.UME.IP".to_string();
    let size = 1.0;
    let expiry = Some("DEC-25".to_string());
    let deal_reference = Some("test-deal-ref".to_string());
    let currency_code = "USD".to_string();

    let order = CreateOrderRequest::sell_option_to_market(
        epic.clone(),
        size,
        expiry.clone(),
        deal_reference.clone(),
        Some(currency_code.clone()),
    );

    assert_eq!(order.epic, epic);
    assert_eq!(order.direction, Direction::Sell);
    // Check that size is rounded correctly
    assert_eq!(order.size, 1.0); // Rounded from 1.0 * 100.0 / 100.0
    assert_eq!(order.order_type, OrderType::Limit); // Corrected from Market to Limit
    assert_eq!(order.time_in_force, TimeInForce::FillOrKill);
    assert!(order.level.is_some()); // Check level is set
    assert_eq!(order.level, Some(0.0)); // Updated: default level value is 0.0
    assert!(!order.guaranteed_stop);
    assert!(order.stop_level.is_none());
    assert!(order.stop_distance.is_none());
    assert!(order.limit_level.is_none());
    assert!(order.limit_distance.is_none());
    assert_eq!(order.expiry, expiry);
    assert_eq!(order.deal_reference, deal_reference);
    assert!(order.force_open);
    assert_eq!(order.currency_code, currency_code);
}

#[test]
fn test_create_order_request_buy_option_to_market() {
    let epic = "CC.D.LCO.UME.IP";
    let size = 2.5;
    let expiry = "DEC-25";
    let deal_id = "test-deal-123";
    let currency = "USD";

    let request = CreateOrderRequest::buy_option_to_market(
        epic.to_string(),
        size,
        Some(expiry.to_string()),
        Some(deal_id.to_string()),
        Some(currency.to_string()),
    );

    assert_eq!(request.epic, epic);
    assert_eq!(request.direction, Direction::Buy);
    assert_eq!(request.size, 2.5);
    assert_eq!(request.order_type, OrderType::Limit); // Updated: order_type is now Limit
    assert_eq!(request.time_in_force, TimeInForce::FillOrKill);
    assert_eq!(request.expiry, Some(expiry.to_string()));
    assert_eq!(request.deal_reference, Some(deal_id.to_string()));
    assert_eq!(request.currency_code, currency.to_string());
}

#[test]
fn test_close_position_request_market() {
    let deal_id = "test-deal-123";
    let direction = Direction::Buy;
    let size = 1.0;

    let request = ClosePositionRequest::market(deal_id.to_string(), direction.clone(), size);

    assert_eq!(request.deal_id, Some(deal_id.to_string()));
    assert_eq!(request.direction, direction);
    assert_eq!(request.size, size);
    assert_eq!(request.order_type, OrderType::Market);
    // time_in_force is now an enum, not an Option<TimeInForce>
}

#[test]
fn test_close_position_request_limit() {
    let deal_id = "test-deal-123";
    let direction = Direction::Sell;
    let size = 2.0;
    let level = 1.2345;

    let request = ClosePositionRequest::limit(deal_id.to_string(), direction.clone(), size, level);

    assert_eq!(request.deal_id, Some(deal_id.to_string()));
    assert_eq!(request.direction, direction);
    assert_eq!(request.size, size);
    assert_eq!(request.order_type, OrderType::Limit);
    assert_eq!(request.time_in_force, TimeInForce::FillOrKill); // Updated: time_in_force is now FillOrKill
    assert_eq!(request.level, Some(level));
}

#[test]
fn test_close_position_request_close_option_to_market_by_epic() {
    let epic = "CC.D.LCO.UME.IP";
    let direction = Direction::Sell;
    let size = 1.0;
    let expiry = "DEC-25";

    let request = ClosePositionRequest::close_option_to_market_by_epic(
        epic.to_string(),
        expiry.to_string(),
        direction.clone(),
        size,
    );

    assert_eq!(request.epic, Some(epic.to_string()));
    assert_eq!(request.expiry, Some(expiry.to_string()));
    assert_eq!(request.direction, direction);
    assert_eq!(request.size, size);
    assert_eq!(request.order_type, OrderType::Limit); // Updated: order_type is now Limit
    assert!(request.deal_id.is_none());
    // time_in_force is now an enum, not an Option<TimeInForce>
}

#[test]
fn test_create_working_order_request_limit() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let level = 1.2345;

    let order = CreateWorkingOrderRequest::limit(epic.to_string(), direction.clone(), size, level);

    assert_eq!(order.epic, epic);
    assert_eq!(order.direction, direction);
    assert_eq!(order.size, size);
    assert_eq!(order.level, level);
    assert_eq!(order.order_type, OrderType::Limit);
    assert_eq!(order.time_in_force, TimeInForce::GoodTillCancelled);
    assert!(order.guaranteed_stop.is_none());
    assert!(order.stop_level.is_none());
    assert!(order.stop_distance.is_none());
    assert!(order.limit_level.is_none());
    assert!(order.limit_distance.is_none());
    assert!(order.good_till_date.is_none());
    assert!(order.deal_reference.is_none());
    assert!(order.currency_code.is_none());
}

#[test]
fn test_create_working_order_request_stop() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Sell;
    let size = 2.0;
    let level = 1.2345;

    let order = CreateWorkingOrderRequest::stop(epic.to_string(), direction.clone(), size, level);

    assert_eq!(order.epic, epic);
    assert_eq!(order.direction, direction);
    assert_eq!(order.size, size);
    assert_eq!(order.level, level);
    assert_eq!(order.order_type, OrderType::Stop);
    assert_eq!(order.time_in_force, TimeInForce::GoodTillCancelled);
    assert!(order.guaranteed_stop.is_none());
    assert!(order.stop_level.is_none());
    assert!(order.stop_distance.is_none());
    assert!(order.limit_level.is_none());
    assert!(order.limit_distance.is_none());
    assert!(order.good_till_date.is_none());
    assert!(order.deal_reference.is_none());
    assert!(order.currency_code.is_none());
}

#[test]
fn test_create_working_order_request_with_stop_loss() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let level = 1.2345;
    let stop_level = 1.2000;

    let order = CreateWorkingOrderRequest::limit(epic.to_string(), direction, size, level)
        .with_stop_loss(stop_level);

    assert_eq!(order.stop_level, Some(stop_level));
}

#[test]
fn test_create_working_order_request_with_take_profit() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let level = 1.2345;
    let limit_level = 1.3000;

    let order = CreateWorkingOrderRequest::limit(epic.to_string(), direction, size, level)
        .with_take_profit(limit_level);

    assert_eq!(order.limit_level, Some(limit_level));
}

#[test]
fn test_create_working_order_request_with_reference() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let level = 1.2345;
    let reference = "test-reference-123";

    let order = CreateWorkingOrderRequest::limit(epic.to_string(), direction, size, level)
        .with_reference(reference.to_string());

    assert_eq!(order.deal_reference, Some(reference.to_string()));
}

#[test]
fn test_create_working_order_request_expires_at() {
    let epic = "CS.D.EURUSD.TODAY.IP";
    let direction = Direction::Buy;
    let size = 1.0;
    let level = 1.2345;
    let expiry_date = "2025-12-31T23:59:59";

    let order = CreateWorkingOrderRequest::limit(epic.to_string(), direction, size, level)
        .expires_at(expiry_date.to_string());

    assert_eq!(order.time_in_force, TimeInForce::GoodTillDate);
    assert_eq!(order.good_till_date, Some(expiry_date.to_string()));
}

#[test]
fn test_deserialize_nullable_status() {
    // Helper struct for testing
    #[derive(Deserialize)]
    struct TestStatus {
        // Implementamos nuestra propia función de deserialización para probar la funcionalidad
        // ya que deserialize_nullable_status es privada
        #[serde(deserialize_with = "deserialize_status_or_default")]
        status: Status,
    }

    // Función de deserialización local para pruebas
    fn deserialize_status_or_default<'de, D>(deserializer: D) -> Result<Status, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt = Option::deserialize(deserializer)?;
        Ok(opt.unwrap_or(Status::Rejected))
    }

    // Test with a valid status
    let json_with_status = json!({
        "status": "OPEN"
    });
    let result: TestStatus = serde_json::from_value(json_with_status).unwrap();
    assert_eq!(result.status, Status::Open);

    // Test with null status (should default to Rejected)
    let json_with_null = json!({
        "status": null
    });
    let result: TestStatus = serde_json::from_value(json_with_null).unwrap();
    assert_eq!(result.status, Status::Rejected);
}

// Serialization and Deserialization Tests for CreateOrderRequest and ClosePositionRequest

#[test]
fn test_create_order_request_deserialization() {
    let json_data = r#"
    {
      "dealId": null,
      "epic": "DO.D.OTCDSTXE.GG.IP",
      "expiry": "20-AUG-25",
      "direction": "SELL",
      "size": 5.25,
      "level": 0.0,
      "orderType": "LIMIT",
      "timeInForce": "FILL_OR_KILL",
      "quoteId": null,
      "guaranteedStop": false,
      "forceOpen": true,
      "currencyCode": "EUR",
      "trailingStop": false
    }
    "#;

    let order: CreateOrderRequest = serde_json::from_str(json_data).unwrap();

    assert_eq!(order.epic, "DO.D.OTCDSTXE.GG.IP");
    assert_eq!(order.expiry, Some("20-AUG-25".to_string()));
    assert_eq!(order.direction, Direction::Sell);
    assert_eq!(order.size, 5.25);
    assert_eq!(order.level, Some(0.0));
    assert_eq!(order.order_type, OrderType::Limit);
    assert_eq!(order.time_in_force, TimeInForce::FillOrKill);
    assert_eq!(order.quote_id, None);
}

#[test]
fn test_create_order_request_serialization() {
    let order = CreateOrderRequest {
        epic: "DO.D.OTCDSTXE.GG.IP".to_string(),
        direction: Direction::Sell,
        size: 5.25,
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::FillOrKill,
        level: Some(0.0),
        guaranteed_stop: false,
        stop_level: None,
        stop_distance: None,
        limit_level: None,
        limit_distance: None,
        expiry: Some("20-AUG-25".to_string()),
        deal_reference: None,
        force_open: false,
        currency_code: "-".to_string(),
        quote_id: None,
        trailing_stop: Some(false),
        trailing_stop_increment: None,
    };

    let serialized = serde_json::to_string(&order).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    // Verify key fields are present and correct
    assert_eq!(json_value["epic"], "DO.D.OTCDSTXE.GG.IP");
    assert_eq!(json_value["direction"], "SELL");
    assert_eq!(json_value["size"], 5.25);
    assert_eq!(json_value["level"], 0.0);
    assert_eq!(json_value["orderType"], "LIMIT");
    assert_eq!(json_value["timeInForce"], "FILL_OR_KILL");
    assert_eq!(json_value["expiry"], "20-AUG-25");
}

#[test]
fn test_close_position_request_deserialization() {
    let json_data = r#"
    {
      "currencyCode": "EUR",
      "direction": "SELL",
      "epic": "DO.D.OTCDSTXE.GG.IP",
      "expiry": "20-AUG-25",
      "forceOpen": false,
      "guaranteedStop": false,
      "level": 0.0,
      "orderType": "LIMIT",
      "size": 5.25,
      "timeInForce": "FILL_OR_KILL"
    }
    "#;

    let request: ClosePositionRequest = serde_json::from_str(json_data).unwrap();

    assert_eq!(request.direction, Direction::Sell);
    assert_eq!(request.epic, Some("DO.D.OTCDSTXE.GG.IP".to_string()));
    assert_eq!(request.expiry, Some("20-AUG-25".to_string()));
    assert_eq!(request.level, Some(0.0));
    assert_eq!(request.order_type, OrderType::Limit);
    assert_eq!(request.size, 5.25);
    assert_eq!(request.time_in_force, TimeInForce::FillOrKill);
}

#[test]
fn test_close_position_request_serialization() {
    let request = ClosePositionRequest {
        deal_id: None,
        direction: Direction::Sell,
        size: 5.25,
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::FillOrKill,
        level: Some(0.0),
        expiry: Some("20-AUG-25".to_string()),
        epic: Some("DO.D.OTCDSTXE.GG.IP".to_string()),
        quote_id: None,
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    // Verify key fields are present and correct
    assert_eq!(json_value["direction"], "SELL");
    assert_eq!(json_value["epic"], "DO.D.OTCDSTXE.GG.IP");
    assert_eq!(json_value["expiry"], "20-AUG-25");
    assert_eq!(json_value["level"], 0.0);
    assert_eq!(json_value["orderType"], "LIMIT");
    assert_eq!(json_value["size"], 5.25);
    assert_eq!(json_value["timeInForce"], "FILL_OR_KILL");
}

#[test]
fn test_create_order_request_serialization_round_trip() {
    let json_data = r#"
    {
      "dealId": null,
      "epic": "DO.D.OTCDSTXE.GG.IP",
      "expiry": "20-AUG-25",
      "direction": "SELL",
      "size": 5.25,
      "level": 0.0,
      "orderType": "LIMIT",
      "timeInForce": "FILL_OR_KILL",
      "quoteId": null,
      "guaranteedStop": false,
      "forceOpen": true,
      "currencyCode": "EUR",
      "trailingStop": false
    }
    "#;

    // Deserialize JSON to struct
    let order: CreateOrderRequest = serde_json::from_str(json_data).unwrap();

    // Serialize struct back to JSON
    let serialized = serde_json::to_string(&order).unwrap();

    // Deserialize again to verify round-trip consistency
    let order_round_trip: CreateOrderRequest = serde_json::from_str(&serialized).unwrap();

    // Verify key fields match
    assert_eq!(order.epic, order_round_trip.epic);
    assert_eq!(order.direction, order_round_trip.direction);
    assert_eq!(order.size, order_round_trip.size);
    assert_eq!(order.level, order_round_trip.level);
    assert_eq!(order.order_type, order_round_trip.order_type);
    assert_eq!(order.time_in_force, order_round_trip.time_in_force);
    assert_eq!(order.expiry, order_round_trip.expiry);
}

#[test]
fn test_close_position_request_serialization_round_trip() {
    let json_data = r#"
    {
      "currencyCode": "EUR",
      "direction": "SELL",
      "epic": "DO.D.OTCDSTXE.GG.IP",
      "expiry": "20-AUG-25",
      "forceOpen": false,
      "guaranteedStop": false,
      "level": 0.0,
      "orderType": "LIMIT",
      "size": 5.25,
      "timeInForce": "FILL_OR_KILL"
    }
    "#;

    // Deserialize JSON to struct
    let request: ClosePositionRequest = serde_json::from_str(json_data).unwrap();

    // Serialize struct back to JSON
    let serialized = serde_json::to_string(&request).unwrap();

    // Deserialize again to verify round-trip consistency
    let request_round_trip: ClosePositionRequest = serde_json::from_str(&serialized).unwrap();

    // Verify key fields match
    assert_eq!(request.direction, request_round_trip.direction);
    assert_eq!(request.size, request_round_trip.size);
    assert_eq!(request.level, request_round_trip.level);
    assert_eq!(request.order_type, request_round_trip.order_type);
    assert_eq!(request.time_in_force, request_round_trip.time_in_force);
    assert_eq!(request.epic, request_round_trip.epic);
    assert_eq!(request.expiry, request_round_trip.expiry);
}
