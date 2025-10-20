/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use crate::prelude::{Account, MarketDetails};
use crate::presentation::instrument::InstrumentType;
use crate::presentation::market::{
    HistoricalPrice, MarketData, MarketNavigationNode, MarketNode, PriceAllowance,
};
use crate::utils::parsing::deserialize_null_as_empty_vec;
use chrono::{DateTime, Utc};
use pretty_simple_display::{DebugPretty, DisplaySimple};
use serde::{Deserialize, Serialize};

#[derive(
    DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default,
)]
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
        use prettytable::format;
        use prettytable::{Cell, Row, Table};

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
            a.instrument
                .name
                .to_lowercase()
                .cmp(&b.instrument.name.to_lowercase())
        });

        // Add rows
        for details in &sorted_details {
            let bid = details
                .snapshot
                .bid
                .map(|b| format!("{:.2}", b))
                .unwrap_or_else(|| "-".to_string());

            let offer = details
                .snapshot
                .offer
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
                    ed.last_dealing_date
                        .split('T')
                        .next()
                        .unwrap_or(&ed.last_dealing_date)
                        .to_string()
                })
                .unwrap_or_else(|| {
                    details
                        .instrument
                        .expiry
                        .split('T')
                        .next()
                        .unwrap_or(&details.instrument.expiry)
                        .to_string()
                });

            let high_low = format!(
                "{}/{}",
                details
                    .snapshot
                    .high
                    .map(|h| format!("{:.2}", h))
                    .unwrap_or_else(|| "-".to_string()),
                details
                    .snapshot
                    .low
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

/// Model for historical prices
#[derive(DebugPretty, Clone, Serialize, Deserialize)]
pub struct HistoricalPricesResponse {
    /// List of historical price points
    pub prices: Vec<HistoricalPrice>,
    /// Type of the instrument
    #[serde(rename = "instrumentType")]
    pub instrument_type: InstrumentType,
    /// API usage allowance information
    #[serde(rename = "allowance", skip_serializing_if = "Option::is_none", default)]
    pub allowance: Option<PriceAllowance>,
}

impl HistoricalPricesResponse {
    /// Returns the number of price points in the response
    ///
    /// # Returns
    /// Number of price points
    #[must_use]
    pub fn len(&self) -> usize {
        self.prices.len()
    }

    /// Returns true if the response contains no price points
    ///
    /// # Returns
    /// True if empty, false otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.prices.is_empty()
    }

    /// Returns a reference to the prices vector
    ///
    /// # Returns
    /// Reference to the vector of historical prices
    #[must_use]
    pub fn prices(&self) -> &Vec<HistoricalPrice> {
        &self.prices
    }

    /// Returns an iterator over the prices
    ///
    /// # Returns
    /// Iterator over historical prices
    pub fn iter(&self) -> impl Iterator<Item = &HistoricalPrice> {
        self.prices.iter()
    }
}

impl std::fmt::Display for HistoricalPricesResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use prettytable::format;
        use prettytable::{Cell, Row, Table};

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Add header
        table.add_row(Row::new(vec![
            Cell::new("SNAPSHOT TIME"),
            Cell::new("OPEN BID"),
            Cell::new("OPEN ASK"),
            Cell::new("HIGH BID"),
            Cell::new("HIGH ASK"),
            Cell::new("LOW BID"),
            Cell::new("LOW ASK"),
            Cell::new("CLOSE BID"),
            Cell::new("CLOSE ASK"),
            Cell::new("VOLUME"),
        ]));

        // Add rows
        for price in &self.prices {
            let open_bid = price
                .open_price
                .bid
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let open_ask = price
                .open_price
                .ask
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let high_bid = price
                .high_price
                .bid
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let high_ask = price
                .high_price
                .ask
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let low_bid = price
                .low_price
                .bid
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let low_ask = price
                .low_price
                .ask
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let close_bid = price
                .close_price
                .bid
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let close_ask = price
                .close_price
                .ask
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "-".to_string());

            let volume = price
                .last_traded_volume
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string());

            table.add_row(Row::new(vec![
                Cell::new(&price.snapshot_time),
                Cell::new(&open_bid),
                Cell::new(&open_ask),
                Cell::new(&high_bid),
                Cell::new(&high_ask),
                Cell::new(&low_bid),
                Cell::new(&low_ask),
                Cell::new(&close_bid),
                Cell::new(&close_ask),
                Cell::new(&volume),
            ]));
        }

        // Add summary footer
        writeln!(f, "{}", table)?;
        writeln!(f, "\nSummary:")?;
        writeln!(f, "  Total price points: {}", self.prices.len())?;
        writeln!(f, "  Instrument type: {:?}", self.instrument_type)?;

        if let Some(allowance) = &self.allowance {
            writeln!(
                f,
                "  Remaining allowance: {}",
                allowance.remaining_allowance
            )?;
            writeln!(f, "  Total allowance: {}", allowance.total_allowance)?;
        }

        Ok(())
    }
}

/// Model for market search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSearchResponse {
    /// List of markets matching the search criteria
    pub markets: Vec<MarketData>,
}

impl MarketSearchResponse {
    /// Returns the number of markets in the response
    ///
    /// # Returns
    /// Number of markets
    #[must_use]
    pub fn len(&self) -> usize {
        self.markets.len()
    }

    /// Returns true if the response contains no markets
    ///
    /// # Returns
    /// True if empty, false otherwise
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.markets.is_empty()
    }

    /// Returns a reference to the markets vector
    ///
    /// # Returns
    /// Reference to the vector of markets
    #[must_use]
    pub fn markets(&self) -> &Vec<MarketData> {
        &self.markets
    }

    /// Returns an iterator over the markets
    ///
    /// # Returns
    /// Iterator over markets
    pub fn iter(&self) -> impl Iterator<Item = &MarketData> {
        self.markets.iter()
    }
}

impl std::fmt::Display for MarketSearchResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use prettytable::format;
        use prettytable::{Cell, Row, Table};

        let mut table = Table::new();
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
            Cell::new("TYPE"),
        ]));

        // Sort by instrument name
        let mut sorted_markets = self.markets.clone();
        sorted_markets.sort_by(|a, b| {
            a.instrument_name
                .to_lowercase()
                .cmp(&b.instrument_name.to_lowercase())
        });

        // Add rows
        for market in &sorted_markets {
            let bid = market
                .bid
                .map(|b| format!("{:.4}", b))
                .unwrap_or_else(|| "-".to_string());

            let offer = market
                .offer
                .map(|o| format!("{:.4}", o))
                .unwrap_or_else(|| "-".to_string());

            let mid = match (market.bid, market.offer) {
                (Some(b), Some(o)) => format!("{:.4}", (b + o) / 2.0),
                _ => "-".to_string(),
            };

            let spread = match (market.bid, market.offer) {
                (Some(b), Some(o)) => format!("{:.4}", o - b),
                _ => "-".to_string(),
            };

            // Truncate long names
            let name = if market.instrument_name.len() > 30 {
                format!("{}...", &market.instrument_name[0..27])
            } else {
                market.instrument_name.clone()
            };

            // Extract date from expiry
            let expiry = market
                .expiry
                .split('T')
                .next()
                .unwrap_or(&market.expiry)
                .to_string();

            let instrument_type = format!("{:?}", market.instrument_type);

            table.add_row(Row::new(vec![
                Cell::new(&name),
                Cell::new(&market.epic),
                Cell::new(&bid),
                Cell::new(&offer),
                Cell::new(&mid),
                Cell::new(&spread),
                Cell::new(&expiry),
                Cell::new(&instrument_type),
            ]));
        }

        writeln!(f, "{}", table)?;
        writeln!(f, "\nTotal markets found: {}", self.markets.len())?;

        Ok(())
    }
}

/// Response model for market navigation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketNavigationResponse {
    /// List of navigation nodes at the current level
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub nodes: Vec<MarketNavigationNode>,
    /// List of markets at the current level
    #[serde(default, deserialize_with = "deserialize_null_as_empty_vec")]
    pub markets: Vec<MarketData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountsResponse {
    /// List of accounts owned by the user
    pub accounts: Vec<Account>,
}