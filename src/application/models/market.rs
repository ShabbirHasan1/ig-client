pub(crate) use crate::presentation::InstrumentType;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Model for a market instrument with enhanced deserialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instrument {
    /// Unique identifier for the instrument
    pub epic: String,
    /// Human-readable name of the instrument
    pub name: String,
    /// Expiry date of the instrument
    pub expiry: String,
    /// Size of one contract
    #[serde(rename = "contractSize")]
    pub contract_size: String,
    /// Size of one lot
    #[serde(rename = "lotSize")]
    pub lot_size: Option<f64>,
    /// Upper price limit for the instrument
    #[serde(rename = "highLimitPrice")]
    pub high_limit_price: Option<f64>,
    /// Lower price limit for the instrument
    #[serde(rename = "lowLimitPrice")]
    pub low_limit_price: Option<f64>,
    /// Margin factor for the instrument
    #[serde(rename = "marginFactor")]
    pub margin_factor: Option<f64>,
    /// Unit for the margin factor
    #[serde(rename = "marginFactorUnit")]
    pub margin_factor_unit: Option<String>,
    /// Available currencies for trading this instrument
    pub currencies: Option<Vec<Currency>>,
    #[serde(rename = "valueOfOnePip")]
    /// Value of one pip for this instrument
    pub value_of_one_pip: String,
    /// Type of the instrument
    #[serde(rename = "instrumentType")]
    pub instrument_type: Option<InstrumentType>,
    /// Expiry details including last dealing date
    #[serde(rename = "expiryDetails")]
    pub expiry_details: Option<ExpiryDetails>,
    #[serde(rename = "slippageFactor")]
    /// Slippage factor for the instrument
    pub slippage_factor: Option<StepDistance>,
    #[serde(rename = "limitedRiskPremium")]
    /// Premium for limited risk trades
    pub limited_risk_premium: Option<StepDistance>,
    #[serde(rename = "newsCode")]
    /// Code used for news related to this instrument
    pub news_code: Option<String>,
    #[serde(rename = "chartCode")]
    /// Code used for charting this instrument
    pub chart_code: Option<String>,
}

/// Model for an instrument's currency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Currency {
    /// Currency code (e.g., "USD", "EUR")
    pub code: String,
    /// Currency symbol (e.g., "$", "€")
    pub symbol: Option<String>,
    /// Base exchange rate for the currency
    #[serde(rename = "baseExchangeRate")]
    pub base_exchange_rate: Option<f64>,
    /// Current exchange rate
    #[serde(rename = "exchangeRate")]
    pub exchange_rate: Option<f64>,
    /// Whether this is the default currency for the instrument
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
}

/// Model for market data with enhanced deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDetails {
    /// Detailed information about the instrument
    pub instrument: Instrument,
    /// Current market snapshot with prices
    pub snapshot: MarketSnapshot,
    /// Trading rules for the market
    #[serde(rename = "dealingRules")]
    pub dealing_rules: DealingRules,
}

/// Trading rules for a market with enhanced deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealingRules {
    /// Minimum step distance
    #[serde(rename = "minStepDistance")]
    pub min_step_distance: StepDistance,

    /// Minimum deal size allowed
    #[serde(rename = "minDealSize")]
    pub min_deal_size: StepDistance,

    /// Minimum distance for controlled risk stop
    #[serde(rename = "minControlledRiskStopDistance")]
    pub min_controlled_risk_stop_distance: StepDistance,

    /// Minimum distance for normal stop or limit orders
    #[serde(rename = "minNormalStopOrLimitDistance")]
    pub min_normal_stop_or_limit_distance: StepDistance,

    /// Maximum distance for stop or limit orders
    #[serde(rename = "maxStopOrLimitDistance")]
    pub max_stop_or_limit_distance: StepDistance,

    /// Controlled risk spacing
    #[serde(rename = "controlledRiskSpacing")]
    pub controlled_risk_spacing: StepDistance,

    /// Market order preference setting
    #[serde(rename = "marketOrderPreference")]
    pub market_order_preference: String,

    /// Trailing stops preference setting
    #[serde(rename = "trailingStopsPreference")]
    pub trailing_stops_preference: String,

    #[serde(rename = "maxDealSize")]
    /// Maximum deal size allowed
    pub max_deal_size: Option<f64>,
}

/// Market snapshot with enhanced deserialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    /// Current status of the market (e.g., "OPEN", "CLOSED")
    #[serde(rename = "marketStatus")]
    pub market_status: String,

    /// Net change in price since previous close
    #[serde(rename = "netChange")]
    pub net_change: Option<f64>,

    /// Percentage change in price since previous close
    #[serde(rename = "percentageChange")]
    pub percentage_change: Option<f64>,

    /// Time of the last price update
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,

    /// Delay time in milliseconds for market data
    #[serde(rename = "delayTime")]
    pub delay_time: Option<i64>,

    /// Current bid price
    pub bid: Option<f64>,

    /// Current offer/ask price
    pub offer: Option<f64>,

    /// Highest price of the current trading session
    pub high: Option<f64>,

    /// Lowest price of the current trading session
    pub low: Option<f64>,

    /// Odds for binary markets
    #[serde(rename = "binaryOdds")]
    pub binary_odds: Option<f64>,

    /// Factor for decimal places in price display
    #[serde(rename = "decimalPlacesFactor")]
    pub decimal_places_factor: Option<i64>,

    /// Factor for scaling prices
    #[serde(rename = "scalingFactor")]
    pub scaling_factor: Option<i64>,

    /// Extra spread for controlled risk trades
    #[serde(rename = "controlledRiskExtraSpread")]
    pub controlled_risk_extra_spread: Option<f64>,
}

/// Model for market search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSearchResult {
    /// List of markets matching the search criteria
    pub markets: Vec<MarketData>,
}

/// Basic market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketData {
    /// Unique identifier for the market
    pub epic: String,
    /// Human-readable name of the instrument
    #[serde(rename = "instrumentName")]
    pub instrument_name: String,
    /// Type of the instrument
    #[serde(rename = "instrumentType")]
    pub instrument_type: InstrumentType,
    /// Expiry date of the instrument
    pub expiry: String,
    /// Upper price limit for the market
    #[serde(rename = "highLimitPrice")]
    pub high_limit_price: Option<f64>,
    /// Lower price limit for the market
    #[serde(rename = "lowLimitPrice")]
    pub low_limit_price: Option<f64>,
    /// Current status of the market
    #[serde(rename = "marketStatus")]
    pub market_status: String,
    /// Net change in price since previous close
    #[serde(rename = "netChange")]
    pub net_change: Option<f64>,
    /// Percentage change in price since previous close
    #[serde(rename = "percentageChange")]
    pub percentage_change: Option<f64>,
    /// Time of the last price update
    #[serde(rename = "updateTime")]
    pub update_time: Option<String>,
    /// Time of the last price update in UTC
    #[serde(rename = "updateTimeUTC")]
    pub update_time_utc: Option<String>,
    /// Current bid price
    pub bid: Option<f64>,
    /// Current offer/ask price
    pub offer: Option<f64>,
}

impl Display for MarketData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).unwrap_or_else(|_| "Invalid JSON".to_string());
        write!(f, "{json}")
    }
}

/// Model for historical prices
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Historical price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPrice {
    /// Timestamp of the price data point
    #[serde(rename = "snapshotTime")]
    pub snapshot_time: String,
    /// Opening price for the period
    #[serde(rename = "openPrice")]
    pub open_price: PricePoint,
    /// Highest price for the period
    #[serde(rename = "highPrice")]
    pub high_price: PricePoint,
    /// Lowest price for the period
    #[serde(rename = "lowPrice")]
    pub low_price: PricePoint,
    /// Closing price for the period
    #[serde(rename = "closePrice")]
    pub close_price: PricePoint,
    /// Volume traded during the period
    #[serde(rename = "lastTradedVolume")]
    pub last_traded_volume: Option<i64>,
}

/// Price point with bid, ask and last traded prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Bid price at this point
    pub bid: Option<f64>,
    /// Ask/offer price at this point
    pub ask: Option<f64>,
    /// Last traded price at this point
    #[serde(rename = "lastTraded")]
    pub last_traded: Option<f64>,
}

/// Information about API usage allowance for price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAllowance {
    /// Remaining API calls allowed in the current period
    #[serde(rename = "remainingAllowance")]
    pub remaining_allowance: i64,
    /// Total API calls allowed per period
    #[serde(rename = "totalAllowance")]
    pub total_allowance: i64,
    /// Time until the allowance resets
    #[serde(rename = "allowanceExpiry")]
    pub allowance_expiry: i64,
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

/// Details about instrument expiry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpiryDetails {
    /// The last dealing date and time for the instrument
    #[serde(rename = "lastDealingDate")]
    pub last_dealing_date: String,

    /// Information about settlement
    #[serde(rename = "settlementInfo")]
    pub settlement_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Unit for step distances in trading rules
pub enum StepUnit {
    #[serde(rename = "POINTS")]
    /// Points (price movement units)
    Points,
    #[serde(rename = "PERCENTAGE")]
    /// Percentage value
    Percentage,
    #[serde(rename = "pct")]
    /// Alternative representation for percentage
    Pct,
}

/// A struct to handle the minStepDistance value which can be a complex object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepDistance {
    /// Unit type for the distance
    pub unit: Option<StepUnit>,
    /// Numeric value of the distance
    pub value: Option<f64>,
}

/// Helper function to deserialize null values as empty vectors
#[allow(dead_code)]
fn deserialize_null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Node in the market navigation hierarchy
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketNavigationNode {
    /// Unique identifier for the node
    pub id: String,
    /// Display name of the node
    pub name: String,
}

/// Structure representing a node in the market hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketNode {
    /// Node ID
    pub id: String,
    /// Node name
    pub name: String,
    /// Child nodes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<MarketNode>,
    /// Markets in this node
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub markets: Vec<MarketData>,
}
