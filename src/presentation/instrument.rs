use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "UPPERCASE")]
/// Types of financial instruments available in the IG Markets API
pub enum InstrumentType {
    /// Binary options
    Binary,
    /// Bungee capped instruments
    #[serde(rename = "BUNGEE_CAPPED")]
    BungeeCapped,
    /// Bungee commodities instruments
    #[serde(rename = "BUNGEE_COMMODITIES")]
    BungeeCommodities,
    /// Bungee currencies instruments
    #[serde(rename = "BUNGEE_CURRENCIES")]
    BungeeCurrencies,
    /// Bungee indices instruments
    #[serde(rename = "BUNGEE_INDICES")]
    BungeeIndices,
    /// Commodities instruments
    Commodities,
    /// Currency pairs
    Currencies,
    /// Market indices
    Indices,
    /// Knockouts commodities instruments
    #[serde(rename = "KNOCKOUTS_COMMODITIES")]
    KnockoutsCommodities,
    /// Knockouts currencies instruments
    #[serde(rename = "KNOCKOUTS_CURRENCIES")]
    KnockoutsCurrencies,
    /// Knockouts indices instruments
    #[serde(rename = "KNOCKOUTS_INDICES")]
    KnockoutsIndices,
    /// Knockouts shares instruments
    #[serde(rename = "KNOCKOUTS_SHARES")]
    KnockoutsShares,
    /// Options on commodities
    #[serde(rename = "OPT_COMMODITIES")]
    OptCommodities,
    /// Options on currencies
    #[serde(rename = "OPT_CURRENCIES")]
    OptCurrencies,
    /// Options on indices
    #[serde(rename = "OPT_INDICES")]
    OptIndices,
    /// Options on rates
    #[serde(rename = "OPT_RATES")]
    OptRates,
    /// Options on shares
    #[serde(rename = "OPT_SHARES")]
    OptShares,
    /// Interest rates
    Rates,
    /// Market sectors
    Sectors,
    /// Stocks and shares
    Shares,
    /// Sprint market instruments
    #[serde(rename = "SPRINT_MARKET")]
    SprintMarket,
    /// Test market instruments
    #[serde(rename = "TEST_MARKET")]
    TestMarket,
    /// Unknown instrument type
    Unknown,
    /// Options
    #[default]
    Options,
}
