use ig_client::application::services::Listener;
use ig_client::presentation::AccountData;
use lightstreamer_rs::subscription::{ItemUpdate, SubscriptionListener};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
fn test_account_listener_new() {
    // Create a counter to verify if the callback is called
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    // Create a listener with a callback that increments the counter
    let listener: Listener<AccountData> = Listener::new(move |data: &AccountData| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        // Verify that the AccountData object has the expected fields
        assert!(data.to_string().contains("item_name"));
        Ok(())
    });

    // Create a simulated ItemUpdate to test the callback
    let mut fields = HashMap::new();
    fields.insert("ACCOUNT_ID".to_string(), Some("ABCDEF".to_string()));
    fields.insert("BALANCE".to_string(), Some("1000.0".to_string()));
    fields.insert("DEPOSIT".to_string(), Some("500.0".to_string()));
    fields.insert("PL".to_string(), Some("100.0".to_string()));
    fields.insert("AVAILABLE".to_string(), Some("1100.0".to_string()));

    let item_update = ItemUpdate {
        item_name: Some("ACCOUNT:ABCDEF".to_string()),
        item_pos: 1,
        is_snapshot: true,
        fields,
        changed_fields: HashMap::new(),
    };

    // Call on_item_update which internally calls the callback
    <Listener<AccountData> as SubscriptionListener>::on_item_update(&listener, &item_update);

    // Verify that the callback was called
    assert_eq!(*counter.lock().unwrap(), 1);
}
