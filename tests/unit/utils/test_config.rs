use ig_client::utils::config::{get_env_or_default, get_env_or_none};
use std::env;

#[test]
fn test_get_env_or_default_with_existing_var() {
    unsafe {
        env::set_var("TEST_VAR_STRING", "test_value");
        let result: String = get_env_or_default("TEST_VAR_STRING", "default".to_string());
        assert_eq!(result, "test_value");
        env::remove_var("TEST_VAR_STRING");
    }
}

#[test]
fn test_get_env_or_default_with_missing_var() {
    unsafe {
        env::remove_var("MISSING_VAR");
        let result: String = get_env_or_default("MISSING_VAR", "default".to_string());
        assert_eq!(result, "default");
    }
}

#[test]
fn test_get_env_or_default_with_integer() {
    unsafe {
        env::set_var("TEST_VAR_INT", "42");
        let result: i32 = get_env_or_default("TEST_VAR_INT", 0);
        assert_eq!(result, 42);
        env::remove_var("TEST_VAR_INT");
    }
}

#[test]
fn test_get_env_or_default_with_invalid_parse() {
    unsafe {
        env::set_var("TEST_VAR_INVALID", "not_a_number");
        let result: i32 = get_env_or_default("TEST_VAR_INVALID", 99);
        assert_eq!(result, 99); // Should return default
        env::remove_var("TEST_VAR_INVALID");
    }
}

#[test]
fn test_get_env_or_default_with_bool() {
    unsafe {
        env::set_var("TEST_VAR_BOOL", "true");
        let result: bool = get_env_or_default("TEST_VAR_BOOL", false);
        assert!(result);
        env::remove_var("TEST_VAR_BOOL");
    }
}

#[test]
fn test_get_env_or_none_with_existing_var() {
    unsafe {
        env::set_var("TEST_VAR_OPTION", "123");
        let result: Option<i32> = get_env_or_none("TEST_VAR_OPTION");
        assert_eq!(result, Some(123));
        env::remove_var("TEST_VAR_OPTION");
    }
}

#[test]
fn test_get_env_or_none_with_missing_var() {
    unsafe {
        env::remove_var("MISSING_VAR_OPTION");
        let result: Option<i32> = get_env_or_none("MISSING_VAR_OPTION");
        assert_eq!(result, None);
    }
}

#[test]
fn test_get_env_or_none_with_invalid_parse() {
    unsafe {
        env::set_var("TEST_VAR_INVALID_OPTION", "not_a_number");
        let result: Option<i32> = get_env_or_none("TEST_VAR_INVALID_OPTION");
        assert_eq!(result, None);
        env::remove_var("TEST_VAR_INVALID_OPTION");
    }
}

#[test]
fn test_get_env_or_none_with_float() {
    unsafe {
        env::set_var("TEST_VAR_FLOAT", "3.14");
        let result: Option<f64> = get_env_or_none("TEST_VAR_FLOAT");
        assert_eq!(result, Some(3.14));
        env::remove_var("TEST_VAR_FLOAT");
    }
}

#[test]
fn test_get_env_or_none_with_string() {
    unsafe {
        env::set_var("TEST_VAR_STRING_OPTION", "hello");
        let result: Option<String> = get_env_or_none("TEST_VAR_STRING_OPTION");
        assert_eq!(result, Some("hello".to_string()));
        env::remove_var("TEST_VAR_STRING_OPTION");
    }
}
