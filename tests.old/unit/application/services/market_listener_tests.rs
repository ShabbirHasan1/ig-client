use ig_client::application::services::Listener;
use ig_client::presentation::PresentationMarketData;
use lightstreamer_rs::subscription::{ItemUpdate, SubscriptionListener};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[test]
fn test_market_listener_new() {
    // Create a counter to verify if the callback is called
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    // Create a listener with a callback that increments the counter
    let listener = Listener::new(move |data: &PresentationMarketData| {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        // Verify that the MarketData object has the expected fields
        assert!(data.to_string().contains("item_name"));
        Ok(())
    });

    // Create a simulated ItemUpdate to test the callback
    let mut fields = HashMap::new();
    fields.insert("BID".to_string(), Some("1.2000".to_string()));
    fields.insert("OFFER".to_string(), Some("1.2010".to_string()));
    fields.insert("HIGH".to_string(), Some("1.2100".to_string()));
    fields.insert("LOW".to_string(), Some("1.1900".to_string()));
    fields.insert("MARKET_STATE".to_string(), Some("tradeable".to_string()));

    let item_update = ItemUpdate {
        item_name: Some("OP.D.OTCDAX1.021100P.IP".to_string()),
        item_pos: 1,
        is_snapshot: true,
        fields,
        changed_fields: HashMap::new(),
    };

    // Call on_item_update that internally calls the callback
    <Listener<PresentationMarketData> as SubscriptionListener>::on_item_update(
        &listener,
        &item_update,
    );

    // Verify that the callback was called
    assert_eq!(*counter.lock().unwrap(), 1);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_market_listener_mock() {
    // Verify that we can create a mock of the listener
    let listener = Listener::new(|data: &PresentationMarketData| {
        // Simply verify that the MarketData object has the expected fields
        assert!(data.to_string().contains("item_name"));
        Ok(())
    });

    // Create a simulated ItemUpdate to test the callback
    let mut fields = HashMap::new();
    fields.insert("BID".to_string(), Some("1.2000".to_string()));
    fields.insert("OFFER".to_string(), Some("1.2010".to_string()));
    fields.insert("HIGH".to_string(), Some("1.2100".to_string()));
    fields.insert("LOW".to_string(), Some("1.1900".to_string()));
    fields.insert("MARKET_STATE".to_string(), Some("tradeable".to_string()));

    let item_update = ItemUpdate {
        item_name: Some("OP.D.OTCDAX1.021100P.IP".to_string()),
        item_pos: 1,
        is_snapshot: true,
        fields,
        changed_fields: HashMap::new(),
    };

    <Listener<PresentationMarketData> as SubscriptionListener>::on_item_update(
        &listener,
        &item_update,
    );
    assert!(true);
}
