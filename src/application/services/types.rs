use crate::application::models::market::{MarketData, MarketNode};
use crate::error::AppError;
use crate::impl_json_display;
use crate::presentation::InstrumentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result type for listener operations that don't return a value but may return an error
pub type ListenerResult = Result<(), AppError>;

/// A data structure representing an entry in the database, containing details about a financial trading instrument.
///
/// # Fields
///
/// * `symbol` - The unique identifier or trading symbol for this financial instrument.
/// * `epic` - The Epic identifier as provided by the exchange for this instrument.
/// * `name` - A human-readable name that describes the financial instrument.
/// * `instrument_type` - The classification or type of the financial instrument (e.g., stock, bond, future, etc.).
/// * `exchange` - The name of the exchange where this instrument is traded.
/// * `expiry` - The expiration date and time of the instrument, if applicable.
/// * `last_update` - The `DateTime` indicating when this record was last updated.
///
/// This structure includes traits such as `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`, `Eq`, `Hash`, and `Default`
/// for ease of use, serialization, comparison, and hashing operations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct DBEntry {
    /// The trading symbol identifier
    pub symbol: String,
    /// The Epic identifier used by the exchange
    pub epic: String,
    /// Human-readable name of the instrument
    pub name: String,
    /// Instrument type classification
    pub instrument_type: InstrumentType,
    /// The exchange where this instrument is traded
    pub exchange: String,
    /// Expiration date and time for the instrument
    pub expiry: String,
    /// Timestamp of the last update to this record
    pub last_update: DateTime<Utc>,
}

impl_json_display!(DBEntry);

impl From<MarketNode> for DBEntry {
    fn from(value: MarketNode) -> Self {
        let mut entry = DBEntry::default();
        if !value.markets.is_empty() {
            let market = &value.markets[0];
            entry.symbol = market
                .epic
                .split('.')
                .nth(2)
                .unwrap_or_default()
                .to_string();
            entry.epic = market.epic.clone();
            entry.name = market.instrument_name.clone();
            entry.instrument_type = market.instrument_type;
            entry.exchange = "IG".to_string();
            entry.expiry = market.expiry.clone();
            entry.last_update = Utc::now();
        }
        entry
    }
}

impl From<MarketData> for DBEntry {
    fn from(market: MarketData) -> Self {
        DBEntry {
            symbol: market
                .epic
                .split('.')
                .nth(2)
                .unwrap_or_default()
                .to_string(),
            epic: market.epic.clone(),
            name: market.instrument_name.clone(),
            instrument_type: market.instrument_type,
            exchange: "IG".to_string(),
            expiry: market.expiry.clone(),
            last_update: Utc::now(),
        }
    }
}

impl From<&MarketNode> for DBEntry {
    fn from(value: &MarketNode) -> Self {
        DBEntry::from(value.clone())
    }
}

impl From<&MarketData> for DBEntry {
    fn from(market: &MarketData) -> Self {
        DBEntry::from(market.clone())
    }
}
