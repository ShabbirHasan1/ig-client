/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/10/25
******************************************************************************/
use std::env;
use std::fmt::Debug;
use std::str::FromStr;
use tracing::error;

/// Gets an environment variable or returns a default value if not found or cannot be parsed
///
/// # Arguments
///
/// * `env_var` - The name of the environment variable
/// * `default` - The default value to use if the environment variable is not found or cannot be parsed
///
/// # Returns
///
/// The parsed value of the environment variable or the default value
pub fn get_env_or_default<T: FromStr>(env_var: &str, default: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    match env::var(env_var) {
        Ok(val) => val.parse::<T>().unwrap_or_else(|_| {
            error!("Failed to parse {}: {}, using default", env_var, val);
            default
        }),
        Err(_) => default,
    }
}

/// Gets an environment variable and parses it, returning None if not found or invalid
///
/// # Arguments
/// * `env_var` - Name of the environment variable
///
/// # Returns
/// Parsed value if found and valid, None otherwise
pub fn get_env_or_none<T: FromStr>(env_var: &str) -> Option<T>
where
    <T as FromStr>::Err: Debug,
{
    match env::var(env_var) {
        Ok(val) => val.parse::<T>().ok(),
        Err(_) => None,
    }
}
