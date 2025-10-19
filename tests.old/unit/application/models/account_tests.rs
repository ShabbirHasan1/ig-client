// Unit tests for account.rs

#[cfg(test)]
mod tests {
    use ig_client::application::models::account::{Position, Positions};
    use ig_client::application::models::order::Direction;

    use std::fs;

    // Helper function to load test position from JSON file
    fn load_test_position() -> Position {
        let json_content = fs::read_to_string("tests/unit/application/models/position.json")
            .expect("Failed to read position.json file");
        serde_json::from_str(&json_content).expect("Failed to parse position JSON")
    }

    // Helper function to create a position with a specific epic and size
    fn create_position_with_epic(epic: &str, size: f64, pnl: Option<f64>) -> Position {
        let mut position = load_test_position();
        position.market.epic = epic.to_string();
        position.position.size = size;
        position.pnl = pnl;
        position
    }

    #[test]
    fn test_compact_by_epic_empty() {
        // Test with empty vector
        let positions: Vec<Position> = vec![];
        let result = Positions::compact_by_epic(positions);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compact_by_epic_single_position() {
        // Test with a single position
        let position = load_test_position();
        let epic = position.market.epic.clone();
        let size = position.position.size;
        let pnl = position.pnl;
        let positions = vec![position];

        let result = Positions::compact_by_epic(positions);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].market.epic, epic);
        assert_eq!(result[0].position.size, size);
        assert_eq!(result[0].pnl, pnl);
    }

    #[test]
    fn test_compact_by_epic_multiple_positions_same_epic() {
        // Test with multiple positions with the same epic
        let epic = "OP.D.OTCDAXWK.23650P.IP";
        let position1 = create_position_with_epic(epic, 1.0, Some(-6.0));
        let position2 = create_position_with_epic(epic, 2.0, Some(-12.0));
        let positions = vec![position1, position2];

        let result = Positions::compact_by_epic(positions);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].market.epic, epic);
        assert_eq!(result[0].position.size, 3.0); // 1.0 + 2.0
        assert_eq!(result[0].pnl, Some(-18.0)); // -6.0 + -12.0
    }

    #[test]
    fn test_compact_by_epic_multiple_positions_different_epics() {
        // Test with multiple positions with different epics
        let epic1 = "OP.D.OTCDAXWK.23650P.IP";
        let epic2 = "OP.D.OTCDAXWK.24000C.IP";
        let position1 = create_position_with_epic(epic1, 1.0, Some(-6.0));
        let position2 = create_position_with_epic(epic2, 2.0, Some(10.0));
        let positions = vec![position1, position2];

        let result = Positions::compact_by_epic(positions);

        assert_eq!(result.len(), 2);

        // Sort by epic to ensure consistent order for assertions
        let mut sorted_result = result;
        sorted_result.sort_by(|a, b| a.market.epic.cmp(&b.market.epic));

        assert_eq!(sorted_result[0].market.epic, epic1);
        assert_eq!(sorted_result[0].position.size, 1.0);
        assert_eq!(sorted_result[0].pnl, Some(-6.0));

        assert_eq!(sorted_result[1].market.epic, epic2);
        assert_eq!(sorted_result[1].position.size, 2.0);
        assert_eq!(sorted_result[1].pnl, Some(10.0));
    }

    #[test]
    fn test_compact_by_epic_mixed_positions() {
        // Test with a mix of positions with same and different epics
        let epic1 = "OP.D.OTCDAXWK.23650P.IP";
        let epic2 = "OP.D.OTCDAXWK.24000C.IP";
        let epic3 = "OP.D.OTCDAXWK.24500C.IP";

        let position1 = create_position_with_epic(epic1, 1.0, Some(-6.0));
        let position2 = create_position_with_epic(epic2, 2.0, Some(10.0));
        let position3 = create_position_with_epic(epic1, 3.0, Some(-18.0));
        let position4 = create_position_with_epic(epic3, 1.5, Some(7.5));

        let positions = vec![position1, position2, position3, position4];

        let result = Positions::compact_by_epic(positions);

        assert_eq!(result.len(), 3);

        // Sort by epic to ensure consistent order for assertions
        let mut sorted_result = result;
        sorted_result.sort_by(|a, b| a.market.epic.cmp(&b.market.epic));

        assert_eq!(sorted_result[0].market.epic, epic1);
        assert_eq!(sorted_result[0].position.size, 4.0); // 1.0 + 3.0
        assert_eq!(sorted_result[0].pnl, Some(-24.0)); // -6.0 + -18.0

        assert_eq!(sorted_result[1].market.epic, epic2);
        assert_eq!(sorted_result[1].position.size, 2.0);
        assert_eq!(sorted_result[1].pnl, Some(10.0));

        assert_eq!(sorted_result[2].market.epic, epic3);
        assert_eq!(sorted_result[2].position.size, 1.5);
        assert_eq!(sorted_result[2].pnl, Some(7.5));
    }

    #[test]
    fn test_position_add_same_epic() {
        // Test adding positions with the same epic
        let epic = "OP.D.OTCDAXWK.23650P.IP";
        let position1 = create_position_with_epic(epic, 1.0, Some(-6.0));
        let position2 = create_position_with_epic(epic, 2.0, Some(-12.0));

        let result = position1 + position2;

        assert_eq!(result.market.epic, epic);
        assert_eq!(result.position.size, 3.0); // 1.0 + 2.0
        assert_eq!(result.pnl, Some(-18.0)); // -6.0 + -12.0
    }

    #[test]
    #[should_panic(expected = "Cannot add positions from different markets")]
    fn test_position_add_different_epics() {
        // Test adding positions with different epics - should panic
        let position1 = create_position_with_epic("OP.D.OTCDAXWK.23650P.IP", 1.0, Some(-6.0));
        let position2 = create_position_with_epic("OP.D.OTCDAXWK.24000C.IP", 2.0, Some(10.0));

        let _ = position1 + position2; // This should panic
    }

    #[test]
    fn test_position_add_with_none_pnl() {
        // Test adding positions where one has None pnl
        let epic = "OP.D.OTCDAXWK.23650P.IP";
        let position1 = create_position_with_epic(epic, 1.0, Some(-6.0));
        let position2 = create_position_with_epic(epic, 2.0, None);

        let result = position1 + position2;

        assert_eq!(result.market.epic, epic);
        assert_eq!(result.position.size, 3.0); // 1.0 + 2.0
        assert_eq!(result.pnl, Some(-6.0)); // Only position1 has pnl
    }

    #[test]
    fn test_position_details_add_same_direction() {
        // Test adding position details with the same direction
        let position = load_test_position();
        let details1 = position.position.clone();

        let mut details2 = details1.clone();
        details2.size = 2.0;
        details2.contract_size = 2.0;

        let result = details1 + details2;

        assert_eq!(result.contract_size, 3.0); // 1.0 + 2.0
        assert_eq!(result.size, 3.0); // 1.0 + 2.0
        assert_eq!(result.level, 62.2); // Average level (both are the same)
        assert_eq!(result.direction, Direction::Sell); // Direction should be the same
    }

    #[test]
    fn test_position_details_add_opposite_direction() {
        // Test adding position details with opposite directions
        let position = load_test_position();
        let details1 = position.position.clone();

        let mut details2 = details1.clone();
        details2.size = 2.0;
        details2.contract_size = 2.0;
        details2.direction = Direction::Buy; // Opposite direction

        let result = details1 + details2;

        assert_eq!(result.contract_size, 1.0); // |1.0 - 2.0| = 1.0
        assert_eq!(result.size, 1.0); // |1.0 - 2.0| = 1.0
        assert_eq!(result.level, 62.2); // Average level (both are the same)
        assert_eq!(result.direction, Direction::Sell); // Direction from details1
    }

    #[test]
    fn test_position_details_add_with_optional_fields() {
        // Test adding position details with different optional fields
        let position = load_test_position();
        let mut details1 = position.position.clone();
        details1.limit_level = Some(70.0);
        details1.stop_level = None;

        let mut details2 = details1.clone();
        details2.limit_level = None;
        details2.stop_level = Some(50.0);

        let result = details1 + details2;

        assert_eq!(result.limit_level, Some(70.0)); // Takes from details1
        assert_eq!(result.stop_level, Some(50.0)); // Takes from details2
    }
}
