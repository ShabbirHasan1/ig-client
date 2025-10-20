use ig_client::storage::utils::{deserialize_from_json, serialize_to_json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestStruct {
    name: String,
    value: i32,
}

#[test]
fn test_serialize_to_json() {
    let test_data = TestStruct {
        name: "test".to_string(),
        value: 42,
    };

    let json = serialize_to_json(&test_data).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("42"));
}

#[test]
fn test_serialize_to_json_empty_string() {
    let test_data = TestStruct {
        name: "".to_string(),
        value: 0,
    };

    let json = serialize_to_json(&test_data).unwrap();
    assert!(json.contains("\"name\":\"\""));
}

#[test]
fn test_serialize_to_json_special_chars() {
    let test_data = TestStruct {
        name: "test\"with\\quotes".to_string(),
        value: 123,
    };

    let json = serialize_to_json(&test_data).unwrap();
    assert!(json.contains("test"));
}

#[test]
fn test_deserialize_from_json() {
    let json = r#"{"name":"test","value":42}"#;

    let result: TestStruct = deserialize_from_json(json).unwrap();
    assert_eq!(result.name, "test");
    assert_eq!(result.value, 42);
}

#[test]
fn test_deserialize_from_json_with_whitespace() {
    let json = r#"{
        "name": "test",
        "value": 42
    }"#;

    let result: TestStruct = deserialize_from_json(json).unwrap();
    assert_eq!(result.name, "test");
    assert_eq!(result.value, 42);
}

#[test]
fn test_deserialize_from_json_invalid() {
    let json = r#"{"invalid json"#;

    let result: Result<TestStruct, _> = deserialize_from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_serialize_deserialize_roundtrip() {
    let original = TestStruct {
        name: "roundtrip".to_string(),
        value: 999,
    };

    let json = serialize_to_json(&original).unwrap();
    let deserialized: TestStruct = deserialize_from_json(&json).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_serialize_vec() {
    let data = vec![
        TestStruct {
            name: "first".to_string(),
            value: 1,
        },
        TestStruct {
            name: "second".to_string(),
            value: 2,
        },
    ];

    let json = serialize_to_json(&data).unwrap();
    assert!(json.contains("first"));
    assert!(json.contains("second"));
}

#[test]
fn test_deserialize_vec() {
    let json = r#"[{"name":"first","value":1},{"name":"second","value":2}]"#;

    let result: Vec<TestStruct> = deserialize_from_json(json).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "first");
    assert_eq!(result[1].name, "second");
}

// Note: Tests for create_database_config_from_env are complex because:
// 1. dotenv() loads .env file which may set DATABASE_URL
// 2. Environment variables persist across tests
// 3. Tests run in parallel making env manipulation unreliable
// These are better tested with integration tests or by mocking

#[test]
fn test_serialize_option_some() {
    #[derive(Serialize)]
    struct TestOption {
        value: Option<String>,
    }

    let data = TestOption {
        value: Some("test".to_string()),
    };

    let json = serialize_to_json(&data).unwrap();
    assert!(json.contains("test"));
}

#[test]
fn test_serialize_option_none() {
    #[derive(Serialize)]
    struct TestOption {
        value: Option<String>,
    }

    let data = TestOption { value: None };

    let json = serialize_to_json(&data).unwrap();
    assert!(json.contains("null"));
}

#[test]
fn test_deserialize_option_some() {
    #[derive(Deserialize, Debug)]
    struct TestOption {
        value: Option<String>,
    }

    let json = r#"{"value":"test"}"#;
    let result: TestOption = deserialize_from_json(json).unwrap();
    assert_eq!(result.value, Some("test".to_string()));
}

#[test]
fn test_deserialize_option_none() {
    #[derive(Deserialize, Debug)]
    struct TestOption {
        value: Option<String>,
    }

    let json = r#"{"value":null}"#;
    let result: TestOption = deserialize_from_json(json).unwrap();
    assert_eq!(result.value, None);
}

#[test]
fn test_serialize_nested_struct() {
    #[derive(Serialize)]
    struct Inner {
        id: i32,
    }

    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
        name: String,
    }

    let data = Outer {
        inner: Inner { id: 42 },
        name: "test".to_string(),
    };

    let json = serialize_to_json(&data).unwrap();
    assert!(json.contains("42"));
    assert!(json.contains("test"));
}

// Note: store_transactions and create_connection_pool require a real database
// and are better tested with integration tests
