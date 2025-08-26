use crate::application::models::market::{MarketData, MarketNode};
use crate::error::AppError;
use crate::impl_json_display;
use crate::presentation::InstrumentType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Result type for listener operations that don't return a value but may return an error
pub type ListenerResult = Result<(), AppError>;

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
        let mut entry = DBEntry::default();
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
        entry
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
