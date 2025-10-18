mod account;
mod chart;
mod instrument;
mod market;
mod price;
/// Module containing serialization and deserialization utilities for working with the IG Markets API
pub mod serialization;
/// Trade-related presentation module containing data structures for trade updates.
pub mod trade;

pub use account::AccountData;
pub use chart::ChartData;
pub use instrument::InstrumentType;
pub use market::{
    MarketFields, MarketState, PresentationMarketData, build_market_hierarchy,
    extract_markets_from_hierarchy,
};
pub use price::PriceData;
pub use trade::TradeData;
