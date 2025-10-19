use ig_client::application::services::Listener;
use ig_client::presentation::ChartData;
use lightstreamer_rs::subscription::{ItemUpdate, SubscriptionListener};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
fn test_chart_listener_new() {
    // Create a counter to verify if the callback is called
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    // Create a listener with a callback that increments the counter
    let listener = Listener::new(move |data: &ChartData| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        // Verify that the ChartData object has the expected fields
        assert!(data.to_string().contains("item_name"));
        Ok(())
    });

    // Create a simulated ItemUpdate to test the callback
    let mut fields = HashMap::new();
    fields.insert("BID".to_string(), Some("1.2000".to_string()));
    fields.insert("OFR".to_string(), Some("1.2010".to_string()));
    fields.insert("LTV".to_string(), Some("1000".to_string()));

    let item_update = ItemUpdate {
        item_name: Some("CHART:OP.D.OTCDAX1.021100P.IP:TICK".to_string()),
        item_pos: 1,
        is_snapshot: true,
        fields,
        changed_fields: HashMap::new(),
    };

    // Call on_item_update which internally calls the callback
    <Listener<ChartData> as SubscriptionListener>::on_item_update(&listener, &item_update);

    // Verify that the callback was called
    assert_eq!(*counter.lock().unwrap(), 1);
}
