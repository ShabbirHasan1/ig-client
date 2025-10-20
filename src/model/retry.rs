/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/10/25
******************************************************************************/
use crate::utils::config::get_env_or_none;

/// Configuration for HTTP request retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries on rate limit (None = infinite retries)
    pub max_retry_count: Option<u32>,
    /// Delay in seconds between retries (None = use default 10 seconds)
    pub retry_delay_secs: Option<u64>,
}

impl RetryConfig {
    /// Creates a new retry configuration with infinite retries and 10 second delay
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new retry configuration with infinite retries and 10 second delay
    #[must_use]
    pub fn infinite() -> Self {
        Self {
            max_retry_count: None,  // infinite retries
            retry_delay_secs: None, // use default 10 seconds
        }
    }

    /// Creates a new retry configuration with a maximum number of retries
    #[must_use]
    pub fn with_max_retries(max_retries: u32) -> Self {
        Self {
            max_retry_count: Some(max_retries),
            retry_delay_secs: None, // use default 10 seconds
        }
    }

    /// Creates a new retry configuration with custom delay
    #[must_use]
    pub fn with_delay(delay_secs: u64) -> Self {
        Self {
            max_retry_count: None, // infinite retries
            retry_delay_secs: Some(delay_secs),
        }
    }

    /// Creates a new retry configuration with both max retries and custom delay
    #[must_use]
    pub fn with_max_retries_and_delay(max_retries: u32, delay_secs: u64) -> Self {
        Self {
            max_retry_count: Some(max_retries),
            retry_delay_secs: Some(delay_secs),
        }
    }

    /// Gets the maximum retry count (0 = infinite)
    #[must_use]
    pub fn max_retries(&self) -> u32 {
        self.max_retry_count.unwrap_or(0)
    }

    /// Gets the retry delay in seconds (default: 10)
    #[must_use]
    pub fn delay_secs(&self) -> u64 {
        self.retry_delay_secs.unwrap_or(10)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        let max_retry_count: Option<u32> = get_env_or_none("MAX_RETRY_COUNT");
        let retry_delay_secs: Option<u64> = get_env_or_none("RETRY_DELAY_SECS");
        
        Self {
            max_retry_count,
            retry_delay_secs,
        }
    }
}