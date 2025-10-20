/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use chrono::{DateTime, Utc};
use pretty_simple_display::{DebugPretty, DisplaySimple};
use serde::{Deserialize, Serialize};
use crate::prelude::MarketDetails;
use crate::presentation::instrument::InstrumentType;
use crate::presentation::market::{MarketData, MarketNode};

#[derive(DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct DBEntryResponse {
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

impl From<MarketNode> for DBEntryResponse {
    fn from(value: MarketNode) -> Self {
        let mut entry = DBEntryResponse::default();
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

impl From<MarketData> for DBEntryResponse {
    fn from(market: MarketData) -> Self {
        DBEntryResponse {
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

impl From<&MarketNode> for DBEntryResponse {
    fn from(value: &MarketNode) -> Self {
        DBEntryResponse::from(value.clone())
    }
}

impl From<&MarketData> for DBEntryResponse {
    fn from(market: &MarketData) -> Self {
        DBEntryResponse::from(market.clone())
    }
}


#[derive(DebugPretty, Clone, Serialize, Deserialize, Default)]
pub struct MultipleMarketDetailsResponse {
    #[serde(rename = "marketDetails")]
    pub market_details: Vec<MarketDetails>,
}

impl MultipleMarketDetailsResponse {
    /// Returns the number of market details in the response
    ///
    /// # Returns
    /// Number of market details
    #[must_use]
    pub fn len(&self) -> usize {
        self.market_details.len()
    }

    /// Returns true if the response contains no market details
    ///
    /// # Returns
    /// True if empty, false otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.market_details.is_empty()
    }

    /// Returns a reference to the market details vector
    ///
    /// # Returns
    /// Reference to the vector of market details
    #[must_use]
    pub fn market_details(&self) -> &Vec<MarketDetails> {
        &self.market_details
    }

    /// Returns an iterator over the market details
    ///
    /// # Returns
    /// Iterator over market details
    pub fn iter(&self) -> impl Iterator<Item = &MarketDetails> {
        self.market_details.iter()
    }
}

impl std::fmt::Display for MultipleMarketDetailsResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use prettytable::{Cell, Row, Table};
        use prettytable::format;

        let mut table = Table::new();
        
        // Set table format
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Add header
        table.add_row(Row::new(vec![
            Cell::new("INSTRUMENT NAME"),
            Cell::new("EPIC"),
            Cell::new("BID"),
            Cell::new("OFFER"),
            Cell::new("MID"),
            Cell::new("SPREAD"),
            Cell::new("EXPIRY"),
            Cell::new("HIGH/LOW"),
        ]));

        // Sort by instrument name
        let mut sorted_details = self.market_details.clone();
        sorted_details.sort_by(|a, b| {
            a.instrument.name.to_lowercase().cmp(&b.instrument.name.to_lowercase())
        });

        // Add rows
        for details in &sorted_details {
            let bid = details.snapshot.bid
                .map(|b| format!("{:.2}", b))
                .unwrap_or_else(|| "-".to_string());
            
            let offer = details.snapshot.offer
                .map(|o| format!("{:.2}", o))
                .unwrap_or_else(|| "-".to_string());
            
            let mid = match (details.snapshot.bid, details.snapshot.offer) {
                (Some(b), Some(o)) => format!("{:.2}", (b + o) / 2.0),
                _ => "-".to_string(),
            };
            
            let spread = match (details.snapshot.bid, details.snapshot.offer) {
                (Some(b), Some(o)) => format!("{:.2}", o - b),
                _ => "-".to_string(),
            };
            
            // Use expiry directly (shorter than last_dealing_date)
            let expiry = details
                .instrument
                .expiry_details
                .as_ref()
                .map(|ed| {
                    // Extract just the date part (YYYY-MM-DD)
                    ed.last_dealing_date.split('T').next().unwrap_or(&ed.last_dealing_date).to_string()
                })
                .unwrap_or_else(|| {
                    details.instrument.expiry.split('T').next().unwrap_or(&details.instrument.expiry).to_string()
                });
            
            let high_low = format!(
                "{}/{}",
                details.snapshot.high
                    .map(|h| format!("{:.2}", h))
                    .unwrap_or_else(|| "-".to_string()),
                details.snapshot.low
                    .map(|l| format!("{:.2}", l))
                    .unwrap_or_else(|| "-".to_string())
            );

            // Truncate long names to make room for EPIC
            let name = if details.instrument.name.len() > 30 {
                format!("{}...", &details.instrument.name[0..27])
            } else {
                details.instrument.name.clone()
            };

            // Don't truncate EPIC - show it complete
            let epic = details.instrument.epic.clone();

            table.add_row(Row::new(vec![
                Cell::new(&name),
                Cell::new(&epic),
                Cell::new(&bid),
                Cell::new(&offer),
                Cell::new(&mid),
                Cell::new(&spread),
                Cell::new(&expiry),
                Cell::new(&high_low),
            ]));
        }

        write!(f, "{}", table)
    }
}
