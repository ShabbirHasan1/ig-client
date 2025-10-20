use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TestStructBool {
    #[serde(
        deserialize_with = "ig_client::presentation::serialization::string_as_bool_opt::deserialize"
    )]
    value: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TestStructFloat {
    #[serde(
        deserialize_with = "ig_client::presentation::serialization::string_as_float_opt::deserialize"
    )]
    value: Option<f64>,
}

#[test]
fn test_string_as_bool_opt_true() {
    let json = r#"{"value": "1"}"#;
    let result: TestStructBool = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, Some(true));
}

#[test]
fn test_string_as_bool_opt_false() {
    let json = r#"{"value": "0"}"#;
    let result: TestStructBool = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, Some(false));
}

#[test]
fn test_string_as_bool_opt_null() {
    let json = r#"{"value": null}"#;
    let result: TestStructBool = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, None);
}

#[test]
fn test_string_as_float_opt_valid() {
    let json = r#"{"value": "123.45"}"#;
    let result: TestStructFloat = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, Some(123.45));
}

#[test]
fn test_string_as_float_opt_negative() {
    let json = r#"{"value": "-456.78"}"#;
    let result: TestStructFloat = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, Some(-456.78));
}

#[test]
fn test_string_as_float_opt_null() {
    let json = r#"{"value": null}"#;
    let result: TestStructFloat = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, None);
}

#[test]
fn test_string_as_float_opt_zero() {
    let json = r#"{"value": "0.0"}"#;
    let result: TestStructFloat = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, Some(0.0));
}

#[test]
fn test_string_as_float_opt_scientific() {
    let json = r#"{"value": "1.23e2"}"#;
    let result: TestStructFloat = serde_json::from_str(json).unwrap();
    assert_eq!(result.value, Some(123.0));
}
