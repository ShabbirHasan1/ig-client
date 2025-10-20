use ig_client::model::retry::RetryConfig;

#[test]
fn test_retry_config_new() {
    let config = RetryConfig::new();
    assert_eq!(config.max_retries(), 0); // infinite
    assert_eq!(config.delay_secs(), 10); // default
}

#[test]
fn test_retry_config_infinite() {
    let config = RetryConfig::infinite();
    assert_eq!(config.max_retries(), 0); // infinite
    assert_eq!(config.delay_secs(), 10); // default
}

#[test]
fn test_retry_config_with_max_retries() {
    let config = RetryConfig::with_max_retries(5);
    assert_eq!(config.max_retries(), 5);
    assert_eq!(config.delay_secs(), 10); // default
}

#[test]
fn test_retry_config_with_delay() {
    let config = RetryConfig::with_delay(30);
    assert_eq!(config.max_retries(), 0); // infinite
    assert_eq!(config.delay_secs(), 30);
}

#[test]
fn test_retry_config_with_max_retries_and_delay() {
    let config = RetryConfig::with_max_retries_and_delay(3, 15);
    assert_eq!(config.max_retries(), 3);
    assert_eq!(config.delay_secs(), 15);
}

#[test]
fn test_retry_config_default() {
    let config = RetryConfig::default();
    // Should use environment variables or defaults
    assert!(config.delay_secs() > 0);
}

#[test]
fn test_retry_config_max_retries_getter() {
    let config1 = RetryConfig {
        max_retry_count: Some(10),
        retry_delay_secs: None,
    };
    assert_eq!(config1.max_retries(), 10);

    let config2 = RetryConfig {
        max_retry_count: None,
        retry_delay_secs: None,
    };
    assert_eq!(config2.max_retries(), 0);
}

#[test]
fn test_retry_config_delay_secs_getter() {
    let config1 = RetryConfig {
        max_retry_count: None,
        retry_delay_secs: Some(25),
    };
    assert_eq!(config1.delay_secs(), 25);

    let config2 = RetryConfig {
        max_retry_count: None,
        retry_delay_secs: None,
    };
    assert_eq!(config2.delay_secs(), 10);
}
