use ig_client::application::interfaces::listener::Listener;
use lightstreamer_rs::subscription::{ItemUpdate, SubscriptionListener};
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::sync::{Arc, Mutex};

// Test data structure that implements required traits
#[derive(Debug, Clone)]
struct TestData {
    value: String,
}

impl Display for TestData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestData({})", self.value)
    }
}

impl From<&ItemUpdate> for TestData {
    fn from(update: &ItemUpdate) -> Self {
        TestData {
            value: update.item_name.clone().unwrap_or_default(),
        }
    }
}

#[test]
fn test_listener_new() {
    let _listener = Listener::<TestData>::new(|data| {
        assert!(!data.value.is_empty());
        Ok(())
    });

    // Just verify it was created without panicking
}

#[test]
fn test_listener_on_item_update() {
    let called = Arc::new(Mutex::new(false));
    let called_clone = Arc::clone(&called);

    let listener = Listener::<TestData>::new(move |data| {
        *called_clone.lock().unwrap() = true;
        assert!(!data.value.is_empty());
        Ok(())
    });

    let item_update = ItemUpdate {
        item_name: Some("TEST_ITEM".to_string()),
        item_pos: 1,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    listener.on_item_update(&item_update);
    assert!(*called.lock().unwrap());
}

#[test]
fn test_listener_on_subscription() {
    let mut listener = Listener::<TestData>::new(|_data| Ok(()));

    // This should just log, not panic
    listener.on_subscription();
}

#[test]
fn test_listener_multiple_updates() {
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let listener = Listener::<TestData>::new(move |_data| {
        *counter_clone.lock().unwrap() += 1;
        Ok(())
    });

    let update1 = ItemUpdate {
        item_name: Some("TEST1".to_string()),
        item_pos: 1,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    let update2 = ItemUpdate {
        item_name: Some("TEST2".to_string()),
        item_pos: 2,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    let update3 = ItemUpdate {
        item_name: Some("TEST3".to_string()),
        item_pos: 3,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    listener.on_item_update(&update1);
    listener.on_item_update(&update2);
    listener.on_item_update(&update3);

    assert_eq!(*counter.lock().unwrap(), 3);
}

#[test]
fn test_listener_thread_safety() {
    use std::thread;

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = Arc::clone(&counter);

    let listener = Arc::new(Listener::<TestData>::new(move |_data| {
        *counter_clone.lock().unwrap() += 1;
        Ok(())
    }));

    let mut handles = vec![];

    for i in 0..5 {
        let listener_clone = Arc::clone(&listener);
        let handle = thread::spawn(move || {
            let update = ItemUpdate {
                item_name: Some(format!("THREAD_{}", i)),
                item_pos: i,
                is_snapshot: false,
                fields: HashMap::new(),
                changed_fields: HashMap::new(),
            };
            listener_clone.on_item_update(&update);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(*counter.lock().unwrap(), 5);
}

#[test]
fn test_listener_with_different_data() {
    let values = Arc::new(Mutex::new(Vec::new()));
    let values_clone = Arc::clone(&values);

    let listener = Listener::<TestData>::new(move |data| {
        values_clone.lock().unwrap().push(data.value.clone());
        Ok(())
    });

    let update1 = ItemUpdate {
        item_name: Some("first".to_string()),
        item_pos: 1,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    let update2 = ItemUpdate {
        item_name: Some("second".to_string()),
        item_pos: 2,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    let update3 = ItemUpdate {
        item_name: Some("third".to_string()),
        item_pos: 3,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    listener.on_item_update(&update1);
    listener.on_item_update(&update2);
    listener.on_item_update(&update3);

    let collected = values.lock().unwrap();
    assert_eq!(collected.len(), 3);
    assert_eq!(collected[0], "first");
    assert_eq!(collected[1], "second");
    assert_eq!(collected[2], "third");
}

#[test]
fn test_listener_error_handling() {
    let listener = Listener::<TestData>::new(|_data| {
        Err(ig_client::error::AppError::InvalidInput(
            "Test error".to_string(),
        ))
    });

    let update = ItemUpdate {
        item_name: Some("ERROR_TEST".to_string()),
        item_pos: 1,
        is_snapshot: false,
        fields: HashMap::new(),
        changed_fields: HashMap::new(),
    };

    // Should not panic even with error
    listener.on_item_update(&update);
}
