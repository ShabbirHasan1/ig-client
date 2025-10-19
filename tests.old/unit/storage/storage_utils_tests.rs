use ig_client::storage::utils::{deserialize_from_json, serialize_to_json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestStruct {
    id: u32,
    name: String,
    values: Vec<i32>,
    metadata: HashMap<String, String>,
}

#[test]
fn test_serialize_to_json() {
    // Create a test struct
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());

    let test_struct = TestStruct {
        id: 42,
        name: "Test".to_string(),
        values: vec![1, 2, 3],
        metadata,
    };

    // Serialize to JSON
    let result = serialize_to_json(&test_struct);
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.contains(r#""id":42"#));
    assert!(json.contains(r#""name":"Test""#));
    assert!(json.contains(r#""values":[1,2,3]"#));
    assert!(json.contains(r#""key1":"value1""#));
    assert!(json.contains(r#""key2":"value2""#));
}

#[test]
fn test_deserialize_from_json() {
    // Create a JSON string
    let json = r#"{
        "id": 42,
        "name": "Test",
        "values": [1, 2, 3],
        "metadata": {
            "key1": "value1",
            "key2": "value2"
        }
    }"#;

    // Deserialize from JSON
    let result: Result<TestStruct, _> = deserialize_from_json(json);
    assert!(result.is_ok());

    let test_struct = result.unwrap();
    assert_eq!(test_struct.id, 42);
    assert_eq!(test_struct.name, "Test");
    assert_eq!(test_struct.values, vec![1, 2, 3]);

    let mut expected_metadata = HashMap::new();
    expected_metadata.insert("key1".to_string(), "value1".to_string());
    expected_metadata.insert("key2".to_string(), "value2".to_string());
    assert_eq!(test_struct.metadata, expected_metadata);
}

#[test]
fn test_deserialize_from_json_invalid() {
    // Create an invalid JSON string
    let invalid_json = r#"{
        "id": 42,
        "name": "Test",
        "values": [1, 2, 3],
        "metadata": {
            "key1": "value1",
            "key2": "value2"
        "
    }"#;

    // Deserialize from invalid JSON
    let result: Result<TestStruct, _> = deserialize_from_json(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_serialize_deserialize_roundtrip() {
    // Create a test struct
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());
    metadata.insert("key2".to_string(), "value2".to_string());

    let original = TestStruct {
        id: 42,
        name: "Test".to_string(),
        values: vec![1, 2, 3],
        metadata,
    };

    // Serialize to JSON
    let json = serialize_to_json(&original).unwrap();

    // Deserialize back from JSON
    let deserialized: TestStruct = deserialize_from_json(&json).unwrap();

    // Verify the roundtrip
    assert_eq!(deserialized, original);
}

#[test]
fn test_serialize_complex_types() {
    // Test with Option types
    #[derive(Debug, Serialize, Deserialize)]
    struct OptionTest {
        maybe_value: Option<i32>,
        definitely_value: Option<String>,
    }

    let option_test = OptionTest {
        maybe_value: None,
        definitely_value: Some("Present".to_string()),
    };

    let result = serialize_to_json(&option_test);
    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains(r#""maybe_value":null"#));
    assert!(json.contains(r#""definitely_value":"Present""#));

    // Test with nested structures
    #[derive(Debug, Serialize, Deserialize)]
    struct NestedTest {
        outer_id: u32,
        inner: TestStruct,
    }

    let mut metadata = HashMap::new();
    metadata.insert("nested_key".to_string(), "nested_value".to_string());

    let nested_test = NestedTest {
        outer_id: 100,
        inner: TestStruct {
            id: 42,
            name: "Inner".to_string(),
            values: vec![4, 5, 6],
            metadata,
        },
    };

    let result = serialize_to_json(&nested_test);
    assert!(result.is_ok());
    let json = result.unwrap();
    assert!(json.contains(r#""outer_id":100"#));
    assert!(json.contains(r#""id":42"#));
    assert!(json.contains(r#""name":"Inner""#));
    assert!(json.contains(r#""values":[4,5,6]"#));
    assert!(json.contains(r#""nested_key":"nested_value""#));
}
