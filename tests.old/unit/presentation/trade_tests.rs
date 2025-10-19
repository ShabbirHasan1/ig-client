#[cfg(test)]
mod tests {
    use ig_client::application::models::order::{Direction, OrderType, Status, TimeInForce};
    use ig_client::presentation::TradeData;
    use lightstreamer_rs::subscription::ItemUpdate;
    use std::collections::HashMap;

    fn create_item_update(
        item_name: Option<String>,
        item_pos: usize,
        is_snapshot: bool,
        fields: HashMap<String, Option<String>>,
        changed_fields: HashMap<String, String>,
    ) -> ItemUpdate {
        ItemUpdate {
            item_name,
            item_pos,
            is_snapshot,
            fields,
            changed_fields,
        }
    }

    #[test]
    fn test_direction_serialization() {
        let buy = Direction::Buy;
        let sell = Direction::Sell;

        let buy_json = serde_json::to_string(&buy).unwrap();
        let sell_json = serde_json::to_string(&sell).unwrap();

        assert_eq!(buy_json, "\"BUY\"");
        assert_eq!(sell_json, "\"SELL\"");

        let buy_deserialized: Direction = serde_json::from_str("\"BUY\"").unwrap();
        let sell_deserialized: Direction = serde_json::from_str("\"SELL\"").unwrap();

        assert_eq!(buy_deserialized, Direction::Buy);
        assert_eq!(sell_deserialized, Direction::Sell);
    }

    #[test]
    fn test_status_serialization() {
        let statuses = vec![
            (Status::Amended, "\"AMENDED\""),
            (Status::Deleted, "\"DELETED\""),
            (Status::FullyClosed, "\"FULLY_CLOSED\""),
            (Status::Opened, "\"OPENED\""),
            (Status::PartiallyClosed, "\"PARTIALLY_CLOSED\""),
            (Status::Closed, "\"CLOSED\""),
            (Status::Open, "\"OPEN\""),
            (Status::Updated, "\"UPDATED\""),
            (Status::Accepted, "\"ACCEPTED\""),
            (Status::Rejected, "\"REJECTED\""),
        ];

        for (status, expected_json) in statuses {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: Status = serde_json::from_str(expected_json).unwrap();
            assert_eq!(deserialized, status);
        }
    }

    #[test]
    fn test_order_type_serialization() {
        let limit = OrderType::Limit;
        let stop = OrderType::Stop;

        let limit_json = serde_json::to_string(&limit).unwrap();
        let stop_json = serde_json::to_string(&stop).unwrap();

        assert_eq!(limit_json, "\"LIMIT\"");
        assert_eq!(stop_json, "\"STOP\"");

        let limit_deserialized: OrderType = serde_json::from_str("\"LIMIT\"").unwrap();
        let stop_deserialized: OrderType = serde_json::from_str("\"STOP\"").unwrap();

        assert_eq!(limit_deserialized, OrderType::Limit);
        assert_eq!(stop_deserialized, OrderType::Stop);
    }

    #[test]
    fn test_time_in_force_serialization() {
        let gtc = TimeInForce::GoodTillCancelled;
        let gtd = TimeInForce::GoodTillDate;

        let gtc_json = serde_json::to_string(&gtc).unwrap();
        let gtd_json = serde_json::to_string(&gtd).unwrap();

        assert_eq!(gtc_json, "\"GOOD_TILL_CANCELLED\"");
        assert_eq!(gtd_json, "\"GOOD_TILL_DATE\"");

        let gtc_deserialized: TimeInForce =
            serde_json::from_str("\"GOOD_TILL_CANCELLED\"").unwrap();
        let gtd_deserialized: TimeInForce = serde_json::from_str("\"GOOD_TILL_DATE\"").unwrap();

        assert_eq!(gtc_deserialized, TimeInForce::GoodTillCancelled);
        assert_eq!(gtd_deserialized, TimeInForce::GoodTillDate);
    }

    #[test]
    fn test_from_item_update_with_confirms() {
        let mut fields = HashMap::new();
        fields.insert("CONFIRMS".to_string(), Some("TestConfirm123".to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            1,
            true,
            fields,
            HashMap::new(),
        );

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        assert_eq!(trade_data.item_name, "TestItem");
        assert_eq!(trade_data.item_pos, 1);
        assert!(trade_data.is_snapshot);
        assert_eq!(
            trade_data.fields.confirms,
            Some("TestConfirm123".to_string())
        );
        assert!(trade_data.fields.opu.is_none());
        assert!(trade_data.fields.wou.is_none());
    }

    #[test]
    fn test_from_item_update_with_opu() {
        let opu_json = r#"{
            "dealReference": "REF123",
            "dealId": "DEAL123",
            "direction": "BUY",
            "epic": "OP.D.OTCDAX1.021100P.IP",
            "status": "OPENED",
            "dealStatus": "ACCEPTED",
            "level": "1.1234",
            "size": "1.5",
            "currency": "USD",
            "timestamp": "2023-05-01T12:34:56",
            "channel": "MOBILE",
            "expiry": "2023-12-31",
            "dealIdOrigin": "ORIGIN123"
        }"#;

        let mut fields = HashMap::new();
        fields.insert("OPU".to_string(), Some(opu_json.to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            1,
            true,
            fields,
            HashMap::new(),
        );

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        assert_eq!(trade_data.item_name, "TestItem");
        assert_eq!(trade_data.item_pos, 1);
        assert!(trade_data.is_snapshot);

        let opu = trade_data.fields.opu.unwrap();
        assert_eq!(opu.deal_reference, Some("REF123".to_string()));
        assert_eq!(opu.deal_id, Some("DEAL123".to_string()));
        assert_eq!(opu.direction, Some(Direction::Buy));
        assert_eq!(opu.epic, Some("OP.D.OTCDAX1.021100P.IP".to_string()));
        assert_eq!(opu.status, Some(Status::Opened));
        assert_eq!(opu.deal_status, Some(Status::Accepted));
        assert_eq!(opu.level, Some(1.1234));
        assert_eq!(opu.size, Some(1.5));
        assert_eq!(opu.currency, Some("USD".to_string()));
        assert_eq!(opu.timestamp, Some("2023-05-01T12:34:56".to_string()));
        assert_eq!(opu.channel, Some("MOBILE".to_string()));
        assert_eq!(opu.expiry, Some("2023-12-31".to_string()));
        assert_eq!(opu.deal_id_origin, Some("ORIGIN123".to_string()));
    }

    #[test]
    fn test_from_item_update_with_wou() {
        let wou_json = r#"{
            "dealReference": "REF456",
            "dealId": "DEAL456",
            "direction": "SELL",
            "epic": "CS.D.GBPUSD.CFD.IP",
            "status": "OPEN",
            "dealStatus": "ACCEPTED",
            "level": "1.3456",
            "size": "2.5",
            "currency": "GBP",
            "timestamp": "2023-06-01T10:20:30",
            "channel": "WEB",
            "expiry": "2023-12-31",
            "stopDistance": "0.0050",
            "limitDistance": "0.0070",
            "guaranteedStop": true,
            "orderType": "LIMIT",
            "timeInForce": "GOOD_TILL_DATE",
            "goodTillDate": "2023-07-01T00:00:00"
        }"#;

        let mut fields = HashMap::new();
        fields.insert("WOU".to_string(), Some(wou_json.to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            2,
            false,
            fields,
            HashMap::new(),
        );

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        assert_eq!(trade_data.item_name, "TestItem");
        assert_eq!(trade_data.item_pos, 2);
        assert!(!trade_data.is_snapshot);

        let wou = trade_data.fields.wou.unwrap();
        assert_eq!(wou.deal_reference, Some("REF456".to_string()));
        assert_eq!(wou.deal_id, Some("DEAL456".to_string()));
        assert_eq!(wou.direction, Some(Direction::Sell));
        assert_eq!(wou.epic, Some("CS.D.GBPUSD.CFD.IP".to_string()));
        assert_eq!(wou.status, Some(Status::Open));
        assert_eq!(wou.deal_status, Some(Status::Accepted));
        assert_eq!(wou.level, Some(1.3456));
        assert_eq!(wou.size, Some(2.5));
        assert_eq!(wou.currency, Some("GBP".to_string()));
        assert_eq!(wou.timestamp, Some("2023-06-01T10:20:30".to_string()));
        assert_eq!(wou.channel, Some("WEB".to_string()));
        assert_eq!(wou.expiry, Some("2023-12-31".to_string()));
        assert_eq!(wou.stop_distance, Some(0.0050));
        assert_eq!(wou.limit_distance, Some(0.0070));
        assert_eq!(wou.guaranteed_stop, Some(true));
        assert_eq!(wou.order_type, Some(OrderType::Limit));
        assert_eq!(wou.time_in_force, Some(TimeInForce::GoodTillDate));
        assert_eq!(wou.good_till_date, Some("2023-07-01T00:00:00".to_string()));
    }

    #[test]
    fn test_from_item_update_with_multiple_fields() {
        let opu_json = r#"{
            "dealReference": "REF123",
            "dealId": "DEAL123",
            "direction": "BUY",
            "epic": "OP.D.OTCDAX1.021100P.IP",
            "level": "1.1234",
            "size": "1.5"
        }"#;

        let wou_json = r#"{
            "dealReference": "REF456",
            "dealId": "DEAL456",
            "direction": "SELL",
            "orderType": "LIMIT"
        }"#;

        let mut fields = HashMap::new();
        fields.insert("CONFIRMS".to_string(), Some("TestConfirm123".to_string()));
        fields.insert("OPU".to_string(), Some(opu_json.to_string()));
        fields.insert("WOU".to_string(), Some(wou_json.to_string()));

        let mut changed_fields = HashMap::new();
        changed_fields.insert("OPU".to_string(), opu_json.to_string());

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            3,
            false,
            fields,
            changed_fields,
        );

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        assert_eq!(trade_data.item_name, "TestItem");
        assert_eq!(trade_data.item_pos, 3);
        assert!(!trade_data.is_snapshot);

        // Check fields
        assert_eq!(
            trade_data.fields.confirms,
            Some("TestConfirm123".to_string())
        );
        assert!(trade_data.fields.opu.is_some());
        assert!(trade_data.fields.wou.is_some());

        // Check changed_fields
        assert!(trade_data.changed_fields.confirms.is_none());
        assert!(trade_data.changed_fields.opu.is_some());
        assert!(trade_data.changed_fields.wou.is_none());
    }

    #[test]
    fn test_from_item_update_with_empty_strings() {
        let mut fields = HashMap::new();
        fields.insert("CONFIRMS".to_string(), Some("".to_string()));
        fields.insert("OPU".to_string(), Some("".to_string()));
        fields.insert("WOU".to_string(), Some("".to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            4,
            true,
            fields,
            HashMap::new(),
        );

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        assert_eq!(trade_data.item_name, "TestItem");
        assert_eq!(trade_data.item_pos, 4);
        assert!(trade_data.is_snapshot);

        // None because we use option_string_empty_as_none for confirms
        assert_eq!(trade_data.fields.confirms, None);
        // None because we explicitly check for empty strings in create_trade_fields
        assert!(trade_data.fields.opu.is_none());
        assert!(trade_data.fields.wou.is_none());
    }

    #[test]
    fn test_invalid_json_handling() {
        let mut fields = HashMap::new();
        fields.insert("OPU".to_string(), Some("{invalid json}".to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            5,
            false,
            fields,
            HashMap::new(),
        );

        let result = TradeData::from_item_update(&item_update);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse OPU JSON"));
    }

    #[test]
    fn test_from_trait_implementation() {
        let mut fields = HashMap::new();
        fields.insert("CONFIRMS".to_string(), Some("TestConfirm123".to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            6,
            true,
            fields,
            HashMap::new(),
        );

        // Using the From trait implementation
        let trade_data: TradeData = (&item_update).into();

        assert_eq!(trade_data.item_name, "TestItem");
        assert_eq!(trade_data.item_pos, 6);
        assert!(trade_data.is_snapshot);
        assert_eq!(
            trade_data.fields.confirms,
            Some("TestConfirm123".to_string())
        );
    }

    #[test]
    fn test_display_implementation() {
        let mut fields = HashMap::new();
        fields.insert("CONFIRMS".to_string(), Some("TestConfirm123".to_string()));

        let item_update = create_item_update(
            Some("TestItem".to_string()),
            7,
            true,
            fields,
            HashMap::new(),
        );

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        let display_string = format!("{trade_data}");
        assert!(display_string.contains("\"item_name\":\"TestItem\""));
        assert!(display_string.contains("\"CONFIRMS\":\"TestConfirm123\""));
    }

    #[test]
    fn test_from_item_update_with_missing_fields() {
        let item_update = create_item_update(None, 8, false, HashMap::new(), HashMap::new());

        let trade_data = TradeData::from_item_update(&item_update).unwrap();

        assert_eq!(trade_data.item_name, "");
        assert_eq!(trade_data.item_pos, 8);
        assert!(!trade_data.is_snapshot);
        assert!(trade_data.fields.confirms.is_none());
        assert!(trade_data.fields.opu.is_none());
        assert!(trade_data.fields.wou.is_none());
    }
}
