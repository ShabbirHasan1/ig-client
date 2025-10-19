// Unit tests for working_order.rs

#[cfg(test)]
mod tests {
    use ig_client::application::models::order::{Direction, OrderType, TimeInForce};
    use ig_client::application::models::working_order::{
        CreateWorkingOrderRequest, CreateWorkingOrderResponse,
    };

    #[test]
    fn test_create_working_order_request_limit() {
        // Test the limit constructor
        let epic = "test_epic".to_string();
        let direction = Direction::Buy;
        let size = 1.5;
        let level = 100.0;

        let request =
            CreateWorkingOrderRequest::limit(epic.clone(), direction.clone(), size, level);

        assert_eq!(request.epic, epic);
        assert_eq!(request.direction, direction);
        assert_eq!(request.size, size);
        assert_eq!(request.level, level);
        assert_eq!(request.order_type, OrderType::Limit);
        assert_eq!(request.time_in_force, TimeInForce::GoodTillCancelled);
        assert!(!request.guaranteed_stop);
        assert_eq!(request.currency_code, None);
        assert_eq!(request.expiry, "DFB");
        assert_eq!(request.deal_reference, None);
        assert_eq!(request.stop_level, None);
        assert_eq!(request.stop_distance, None);
        assert_eq!(request.limit_level, None);
        assert_eq!(request.limit_distance, None);
        assert_eq!(request.good_till_date, None);
    }

    #[test]
    fn test_create_working_order_request_stop() {
        // Test the stop constructor
        let epic = "test_epic".to_string();
        let direction = Direction::Sell;
        let size = 2.0;
        let level = 200.0;

        let request = CreateWorkingOrderRequest::stop(epic.clone(), direction.clone(), size, level);

        assert_eq!(request.epic, epic);
        assert_eq!(request.direction, direction);
        assert_eq!(request.size, size);
        assert_eq!(request.level, level);
        assert_eq!(request.order_type, OrderType::Stop);
        assert_eq!(request.time_in_force, TimeInForce::GoodTillCancelled);
        assert!(!request.guaranteed_stop);
        assert_eq!(request.currency_code, None);
        assert_eq!(request.expiry, "DFB");
        assert_eq!(request.deal_reference, None);
        assert_eq!(request.stop_level, None);
        assert_eq!(request.stop_distance, None);
        assert_eq!(request.limit_level, None);
        assert_eq!(request.limit_distance, None);
        assert_eq!(request.good_till_date, None);
    }

    #[test]
    fn test_with_reference() {
        // Test the with_reference method
        let epic = "test_epic".to_string();
        let direction = Direction::Buy;
        let size = 1.5;
        let level = 100.0;
        let reference = "test_reference".to_string();

        let request = CreateWorkingOrderRequest::limit(epic, direction.clone(), size, level)
            .with_reference(reference.clone());

        assert_eq!(request.deal_reference, Some(reference));
    }

    #[test]
    fn test_with_stop_loss() {
        // Test the with_stop_loss method
        let epic = "test_epic".to_string();
        let direction = Direction::Buy;
        let size = 1.5;
        let level = 100.0;
        let stop_level = 95.0;

        let request = CreateWorkingOrderRequest::limit(epic, direction.clone(), size, level)
            .with_stop_loss(stop_level);

        assert_eq!(request.stop_level, Some(stop_level));
    }

    #[test]
    fn test_with_take_profit() {
        // Test the with_take_profit method
        let epic = "test_epic".to_string();
        let direction = Direction::Buy;
        let size = 1.5;
        let level = 100.0;
        let limit_level = 105.0;

        let request = CreateWorkingOrderRequest::limit(epic, direction.clone(), size, level)
            .with_take_profit(limit_level);

        assert_eq!(request.limit_level, Some(limit_level));
    }

    #[test]
    fn test_expires_at() {
        // Test the expires_at method
        let epic = "test_epic".to_string();
        let direction = Direction::Buy;
        let size = 1.5;
        let level = 100.0;
        let date = "2025-12-31T23:59:59".to_string();

        let request = CreateWorkingOrderRequest::limit(epic, direction.clone(), size, level)
            .expires_at(date.clone());

        assert_eq!(request.time_in_force, TimeInForce::GoodTillDate);
        assert_eq!(request.good_till_date, Some(date));
    }

    #[test]
    fn test_with_expiry() {
        // Test the with_expiry method
        let epic = "test_epic".to_string();
        let direction = Direction::Buy;
        let size = 1.5;
        let level = 100.0;
        let expiry = "DEC-25".to_string();

        let request = CreateWorkingOrderRequest::limit(epic, direction.clone(), size, level)
            .with_expiry(expiry.clone());

        assert_eq!(request.expiry, expiry);
    }

    #[test]
    fn test_create_working_order_response() {
        // Test the CreateWorkingOrderResponse struct
        let deal_reference = "test_reference".to_string();
        let response = CreateWorkingOrderResponse {
            deal_reference: deal_reference.clone(),
        };

        assert_eq!(response.deal_reference, deal_reference);
    }
}
