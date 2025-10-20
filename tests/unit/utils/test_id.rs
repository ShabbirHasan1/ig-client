use ig_client::utils::id::get_id;

#[test]
fn test_get_id_returns_some() {
    let id = get_id();
    assert!(id.is_some());
}

#[test]
fn test_get_id_length() {
    let id = get_id().unwrap();
    assert_eq!(id.len(), 30);
}

#[test]
fn test_get_id_contains_valid_chars() {
    let id = get_id().unwrap();
    let valid_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

    for c in id.chars() {
        assert!(valid_chars.contains(c), "Invalid character: {}", c);
    }
}

#[test]
fn test_get_id_uniqueness() {
    let id1 = get_id().unwrap();
    let id2 = get_id().unwrap();

    // IDs should be different (extremely high probability)
    assert_ne!(id1, id2);
}

#[test]
fn test_get_id_multiple_calls() {
    let mut ids = std::collections::HashSet::new();

    // Generate 100 IDs and ensure they're all unique
    for _ in 0..100 {
        let id = get_id().unwrap();
        assert!(ids.insert(id), "Duplicate ID generated");
    }

    assert_eq!(ids.len(), 100);
}
