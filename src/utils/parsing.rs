use crate::presentation::order::Status;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use tracing::warn;

/// Structure to represent the parsed option information from an instrument name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedOptionInfo {
    /// Name of the underlying asset (e.g., "US Tech 100")
    pub asset_name: String,
    /// Strike price of the option (e.g., "19200")
    pub strike: Option<String>,
    /// Type of the option: CALL or PUT
    pub option_type: Option<String>,
}

impl fmt::Display for ParsedOptionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Asset: {}, Strike: {}, Type: {}",
            self.asset_name,
            self.strike.as_deref().unwrap_or("N/A"),
            self.option_type.as_deref().unwrap_or("N/A")
        )
    }
}

/// Structure to represent the parsed market data with additional information
#[derive(Debug, Serialize)]
pub struct ParsedMarketData {
    /// Unique identifier for the market (EPIC code)
    pub epic: String,
    /// Full name of the financial instrument
    pub instrument_name: String,
    /// Expiry date of the instrument (if applicable)
    pub expiry: String,
    /// Name of the underlying asset
    pub asset_name: String,
    /// Strike price for options
    pub strike: Option<String>,
    /// Type of option (e.g., 'CALL' or 'PUT')
    pub option_type: Option<String>,
}

/// Normalize text by removing accents and standardizing names
///
/// This function converts accented characters to their non-accented equivalents
/// and standardizes certain names (e.g., "Japan" in different languages)
pub fn normalize_text(text: &str) -> String {
    // Special case for Japan in Spanish
    if text.contains("Japón") {
        return text.replace("Japón", "Japan");
    }

    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' => result.push('a'),
            'é' | 'è' | 'ë' | 'ê' => result.push('e'),
            'í' | 'ì' | 'ï' | 'î' => result.push('i'),
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => result.push('o'),
            'ú' | 'ù' | 'ü' | 'û' => result.push('u'),
            'ñ' => result.push('n'),
            'ç' => result.push('c'),
            'Á' | 'À' | 'Ä' | 'Â' | 'Ã' => result.push('A'),
            'É' | 'È' | 'Ë' | 'Ê' => result.push('E'),
            'Í' | 'Ì' | 'Ï' | 'Î' => result.push('I'),
            'Ó' | 'Ò' | 'Ö' | 'Ô' | 'Õ' => result.push('O'),
            'Ú' | 'Ù' | 'Ü' | 'Û' => result.push('U'),
            'Ñ' => result.push('N'),
            'Ç' => result.push('C'),
            _ => result.push(c),
        }
    }
    result
}

/// Parse the instrument name to extract asset name, strike price, and option type
///
/// # Examples
///
/// ```
/// use ig_client::utils::parsing::parse_instrument_name;
///
/// let info = parse_instrument_name("US Tech 100 19200 CALL ($1)");
/// assert_eq!(info.asset_name, "US Tech 100");
/// assert_eq!(info.strike, Some("19200".to_string()));
/// assert_eq!(info.option_type, Some("CALL".to_string()));
///
/// let info = parse_instrument_name("Germany 40");
/// assert_eq!(info.asset_name, "Germany 40");
/// assert_eq!(info.strike, None);
/// assert_eq!(info.option_type, None);
/// ```
pub fn parse_instrument_name(instrument_name: &str) -> ParsedOptionInfo {
    // Create regex patterns for different instrument name formats
    // Lazy initialization of regex patterns
    lazy_static::lazy_static! {
        // Pattern for standard options like "US Tech 100 19200 CALL ($1)"
        static ref OPTION_PATTERN: Regex = Regex::new(r"^(.*?)\s+(\d+(?:\.\d+)?)\s+(CALL|PUT)(?:\s+\(.*?\))?$").unwrap();

        // Pattern for options with decimal strikes like "Volatility Index 10.5 PUT ($1)"
        static ref DECIMAL_OPTION_PATTERN: Regex = Regex::new(r"^(.*?)\s+(\d+\.\d+)\s+(CALL|PUT)(?:\s+\(.*?\))?$").unwrap();

        // Pattern for options with no space between parenthesis and strike like "Weekly Germany 40 (Wed)27500 PUT"
        static ref SPECIAL_OPTION_PATTERN: Regex = Regex::new(r"^(.*?)\s+\(([^)]+)\)(\d+)\s+(CALL|PUT)(?:\s+\(.*?\))?$").unwrap();

        // Pattern for options with incomplete parenthesis like "Weekly USDJPY 12950 CALL (Y100"
        static ref INCOMPLETE_PAREN_PATTERN: Regex = Regex::new(r"^(.*?)\s+(\d+(?:\.\d+)?)\s+(CALL|PUT)\s+\([^)]*$").unwrap();

        // Pattern for other instruments that don't follow the option pattern
        static ref GENERIC_PATTERN: Regex = Regex::new(r"^(.*?)(?:\s+\(.*?\))?$").unwrap();

        // Pattern to clean up asset names
        static ref DAILY_WEEKLY_PATTERN: Regex = Regex::new(r"^(Daily|Weekly)\s+(.*?)$").unwrap();
        static ref END_OF_MONTH_PATTERN: Regex = Regex::new(r"^(End of Month)\s+(.*?)$").unwrap();
        static ref QUARTERLY_PATTERN: Regex = Regex::new(r"^(Quarterly)\s+(.*?)$").unwrap();
        static ref MONTHLY_PATTERN: Regex = Regex::new(r"^(Monthly)\s+(.*?)$").unwrap();
        static ref SUFFIX_PATTERN: Regex = Regex::new(r"^(.*?)\s+\(.*?\)$").unwrap();
    }

    // Helper function to clean up asset names
    fn clean_asset_name(asset_name: &str) -> String {
        // First normalize the text to remove accents
        let normalized_name = normalize_text(asset_name);

        // Remove prefixes like "Daily", "Weekly", etc.
        let asset_name = if let Some(captures) = DAILY_WEEKLY_PATTERN.captures(&normalized_name) {
            captures.get(2).unwrap().as_str().trim()
        } else if let Some(captures) = END_OF_MONTH_PATTERN.captures(&normalized_name) {
            captures.get(2).unwrap().as_str().trim()
        } else if let Some(captures) = QUARTERLY_PATTERN.captures(&normalized_name) {
            captures.get(2).unwrap().as_str().trim()
        } else if let Some(captures) = MONTHLY_PATTERN.captures(&normalized_name) {
            captures.get(2).unwrap().as_str().trim()
        } else {
            &normalized_name
        };

        // Remove suffixes like "(End of Month)", etc.
        let asset_name = if let Some(captures) = SUFFIX_PATTERN.captures(asset_name) {
            captures.get(1).unwrap().as_str().trim()
        } else {
            asset_name
        };

        asset_name.to_string()
    }

    if let Some(captures) = OPTION_PATTERN.captures(instrument_name) {
        // This is an option with strike and type
        let asset_name = captures.get(1).unwrap().as_str().trim();
        ParsedOptionInfo {
            asset_name: clean_asset_name(asset_name),
            strike: Some(captures.get(2).unwrap().as_str().to_string()),
            option_type: Some(captures.get(3).unwrap().as_str().to_string()),
        }
    } else if let Some(captures) = SPECIAL_OPTION_PATTERN.captures(instrument_name) {
        // This is a special case like "Weekly Germany 40 (Wed)27500 PUT"
        let base_name = captures.get(1).unwrap().as_str().trim();
        ParsedOptionInfo {
            asset_name: clean_asset_name(base_name),
            strike: Some(captures.get(3).unwrap().as_str().to_string()),
            option_type: Some(captures.get(4).unwrap().as_str().to_string()),
        }
    } else if let Some(captures) = INCOMPLETE_PAREN_PATTERN.captures(instrument_name) {
        // This is a case with incomplete parenthesis like "Weekly USDJPY 12950 CALL (Y100"
        let asset_name = captures.get(1).unwrap().as_str().trim();
        ParsedOptionInfo {
            asset_name: clean_asset_name(asset_name),
            strike: Some(captures.get(2).unwrap().as_str().to_string()),
            option_type: Some(captures.get(3).unwrap().as_str().to_string()),
        }
    } else if let Some(captures) = DECIMAL_OPTION_PATTERN.captures(instrument_name) {
        // This is an option with decimal strike
        let asset_name = captures.get(1).unwrap().as_str().trim();
        ParsedOptionInfo {
            asset_name: clean_asset_name(asset_name),
            strike: Some(captures.get(2).unwrap().as_str().to_string()),
            option_type: Some(captures.get(3).unwrap().as_str().to_string()),
        }
    } else if let Some(captures) = GENERIC_PATTERN.captures(instrument_name) {
        // This is a generic instrument without strike or type
        let asset_name = captures.get(1).unwrap().as_str().trim();
        ParsedOptionInfo {
            asset_name: clean_asset_name(asset_name),
            strike: None,
            option_type: None,
        }
    } else {
        // Fallback for any other format
        warn!("Could not parse instrument name: {}", instrument_name);
        ParsedOptionInfo {
            asset_name: instrument_name.to_string(),
            strike: None,
            option_type: None,
        }
    }
}

/// Helper function to deserialize null values as empty vectors
pub fn deserialize_null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Helper function to deserialize a nullable status field
/// When the status is null in the JSON, we default to Open status
pub fn deserialize_nullable_status<'de, D>(deserializer: D) -> Result<Status, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or(Status::Open))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instrument_name_standard_option() {
        let info = parse_instrument_name("US Tech 100 19200 CALL ($1)");
        assert_eq!(info.asset_name, "US Tech 100");
        assert_eq!(info.strike, Some("19200".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_decimal_strike() {
        let info = parse_instrument_name("Volatility Index 10.5 PUT ($1)");
        assert_eq!(info.asset_name, "Volatility Index");
        assert_eq!(info.strike, Some("10.5".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_no_option() {
        let info = parse_instrument_name("Germany 40");
        assert_eq!(info.asset_name, "Germany 40");
        assert_eq!(info.strike, None);
        assert_eq!(info.option_type, None);
    }

    #[test]
    fn test_parse_instrument_name_with_parenthesis() {
        let info = parse_instrument_name("US 500 (Mini)");
        assert_eq!(info.asset_name, "US 500");
        assert_eq!(info.strike, None);
        assert_eq!(info.option_type, None);
    }

    #[test]
    fn test_parse_instrument_name_special_format() {
        let info = parse_instrument_name("Weekly Germany 40 (Wed)27500 PUT");
        assert_eq!(info.asset_name, "Germany 40");
        assert_eq!(info.strike, Some("27500".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_daily_prefix() {
        let info = parse_instrument_name("Daily Germany 40 24225 CALL");
        assert_eq!(info.asset_name, "Germany 40");
        assert_eq!(info.strike, Some("24225".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_weekly_prefix() {
        let info = parse_instrument_name("Weekly US Tech 100 19200 CALL");
        assert_eq!(info.asset_name, "US Tech 100");
        assert_eq!(info.strike, Some("19200".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_end_of_month_prefix() {
        let info = parse_instrument_name("End of Month EU Stocks 50 4575 PUT");
        assert_eq!(info.asset_name, "EU Stocks 50");
        assert_eq!(info.strike, Some("4575".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_end_of_month_suffix() {
        let info = parse_instrument_name("US 500 (End of Month) 3200 PUT");
        assert_eq!(info.asset_name, "US 500");
        assert_eq!(info.strike, Some("3200".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_quarterly_prefix() {
        let info = parse_instrument_name("Quarterly GBPUSD 10000 PUT ($1)");
        assert_eq!(info.asset_name, "GBPUSD");
        assert_eq!(info.strike, Some("10000".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_weekly_with_day() {
        let info = parse_instrument_name("Weekly Germany 40 (Mon) 18500 PUT");
        assert_eq!(info.asset_name, "Germany 40");
        assert_eq!(info.strike, Some("18500".to_string()));
        assert_eq!(info.option_type, Some("PUT".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_incomplete_parenthesis() {
        let info = parse_instrument_name("Weekly USDJPY 12950 CALL (Y100");
        assert_eq!(info.asset_name, "USDJPY");
        assert_eq!(info.strike, Some("12950".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));
    }

    #[test]
    fn test_parse_instrument_name_with_accents() {
        let info = parse_instrument_name("Japón 225 18500 CALL");
        assert_eq!(info.asset_name, "Japan 225");
        assert_eq!(info.strike, Some("18500".to_string()));
        assert_eq!(info.option_type, Some("CALL".to_string()));
    }
}
