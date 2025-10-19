use crate::presentation::serialization::{string_as_bool_opt, string_as_float_opt};
use lightstreamer_rs::subscription::ItemUpdate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use crate::presentation::instrument::InstrumentType;

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

/// Represents the current state of a market
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum MarketState {
    /// Market is closed for trading
    Closed,
    /// Market is offline and not available
    #[default]
    Offline,
    /// Market is open and available for trading
    Tradeable,
    /// Market is in edit mode
    Edit,
    /// Market is in auction phase
    Auction,
    /// Market is in auction phase but editing is not allowed
    AuctionNoEdit,
    /// Market is temporarily suspended
    Suspended,
}

/// Representation of market data received from the IG Markets streaming API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresentationMarketData {
    /// Name of the item this data belongs to
    pub item_name: String,
    /// Position of the item in the subscription
    pub item_pos: i32,
    /// All market fields
    pub fields: MarketFields,
    /// Fields that have changed in this update
    pub changed_fields: MarketFields,
    /// Whether this is a snapshot or an update
    pub is_snapshot: bool,
}

impl PresentationMarketData {
    /// Converts an ItemUpdate from the Lightstreamer API to a MarketData object
    ///
    /// # Arguments
    /// * `item_update` - The ItemUpdate received from the Lightstreamer API
    ///
    /// # Returns
    /// * `Result<Self, String>` - The converted MarketData or an error message
    pub fn from_item_update(item_update: &ItemUpdate) -> Result<Self, String> {
        // Extract the item_name, defaulting to an empty string if None
        let item_name = item_update.item_name.clone().unwrap_or_default();

        // Convert item_pos from usize to i32
        let item_pos = item_update.item_pos as i32;

        // Extract is_snapshot
        let is_snapshot = item_update.is_snapshot;

        // Convert fields
        let fields = Self::create_market_fields(&item_update.fields)?;

        // Convert changed_fields by first creating a HashMap<String, Option<String>>
        let mut changed_fields_map: HashMap<String, Option<String>> = HashMap::new();
        for (key, value) in &item_update.changed_fields {
            changed_fields_map.insert(key.clone(), Some(value.clone()));
        }
        let changed_fields = Self::create_market_fields(&changed_fields_map)?;

        Ok(PresentationMarketData {
            item_name,
            item_pos,
            fields,
            changed_fields,
            is_snapshot,
        })
    }

    /// Helper method to create MarketFields from a HashMap of field values
    ///
    /// # Arguments
    /// * `fields_map` - HashMap containing field names and their string values
    ///
    /// # Returns
    /// * `Result<MarketFields, String>` - The parsed MarketFields or an error message
    fn create_market_fields(
        fields_map: &HashMap<String, Option<String>>,
    ) -> Result<MarketFields, String> {
        // Helper function to safely get a field value
        let get_field = |key: &str| -> Option<String> { fields_map.get(key).cloned().flatten() };

        // Parse market state
        let market_state = match get_field("MARKET_STATE").as_deref() {
            Some("closed") => Some(MarketState::Closed),
            Some("offline") => Some(MarketState::Offline),
            Some("tradeable") => Some(MarketState::Tradeable),
            Some("edit") => Some(MarketState::Edit),
            Some("auction") => Some(MarketState::Auction),
            Some("auction_no_edit") => Some(MarketState::AuctionNoEdit),
            Some("suspended") => Some(MarketState::Suspended),
            Some(unknown) => return Err(format!("Unknown market state: {unknown}")),
            None => None,
        };

        // Parse boolean field
        let market_delay = match get_field("MARKET_DELAY").as_deref() {
            Some("0") => Some(false),
            Some("1") => Some(true),
            Some(val) => return Err(format!("Invalid MARKET_DELAY value: {val}")),
            None => None,
        };

        // Helper function to parse float values
        let parse_float = |key: &str| -> Result<Option<f64>, String> {
            match get_field(key) {
                Some(val) if !val.is_empty() => val
                    .parse::<f64>()
                    .map(Some)
                    .map_err(|_| format!("Failed to parse {key} as float: {val}")),
                _ => Ok(None),
            }
        };

        Ok(MarketFields {
            mid_open: parse_float("MID_OPEN")?,
            high: parse_float("HIGH")?,
            offer: parse_float("OFFER")?,
            change: parse_float("CHANGE")?,
            market_delay,
            low: parse_float("LOW")?,
            bid: parse_float("BID")?,
            change_pct: parse_float("CHANGE_PCT")?,
            market_state,
            update_time: get_field("UPDATE_TIME"),
        })
    }
}

impl fmt::Display for PresentationMarketData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{json}")
    }
}

impl From<&ItemUpdate> for PresentationMarketData {
    fn from(item_update: &ItemUpdate) -> Self {
        Self::from_item_update(item_update).unwrap_or_else(|_| PresentationMarketData {
            item_name: String::new(),
            item_pos: 0,
            fields: MarketFields::default(),
            changed_fields: MarketFields::default(),
            is_snapshot: false,
        })
    }
}

/// Fields containing market price and status information
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MarketFields {
    /// The mid-open price of the market
    #[serde(rename = "MID_OPEN")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub mid_open: Option<f64>,

    /// The highest price reached by the market in the current trading session
    #[serde(rename = "HIGH")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub high: Option<f64>,

    /// The current offer (ask) price of the market
    #[serde(rename = "OFFER")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub offer: Option<f64>,

    /// The absolute price change since the previous close
    #[serde(rename = "CHANGE")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub change: Option<f64>,

    /// Indicates if there is a delay in market data
    #[serde(rename = "MARKET_DELAY")]
    #[serde(with = "string_as_bool_opt")]
    #[serde(default)]
    pub market_delay: Option<bool>,

    /// The lowest price reached by the market in the current trading session
    #[serde(rename = "LOW")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub low: Option<f64>,

    /// The current bid price of the market
    #[serde(rename = "BID")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub bid: Option<f64>,

    /// The percentage price change since the previous close
    #[serde(rename = "CHANGE_PCT")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub change_pct: Option<f64>,

    /// The current state of the market (e.g., Tradeable, Closed, etc.)
    #[serde(rename = "MARKET_STATE")]
    #[serde(default)]
    pub market_state: Option<MarketState>,

    /// The timestamp of the last market update
    #[serde(rename = "UPDATE_TIME")]
    #[serde(default)]
    pub update_time: Option<String>,
}





// pub fn build_market_hierarchy<'a>(
//     market_service: &'a impl MarketService,
//     session: &'a IgSession,
//     node_id: Option<&'a str>,
//     depth: usize,
// ) -> Pin<Box<dyn Future<Output = Result<Vec<MarketNode>, AppError>> + 'a>> {
//     Box::pin(async move {
//         // Limit the depth to avoid infinite loops
//         if depth > 7 {
//             debug!("Reached maximum depth of 5, stopping recursion");
//             return Ok(Vec::new());
//         }
//
//         // Acquire the semaphore to limit concurrency
//         // This ensures that only one API request is made at a time
//         let _permit = API_SEMAPHORE.clone().acquire_owned().await.unwrap();
//
//         // The rate limiter will handle any necessary delays between requests
//         // No explicit sleep calls are needed here
//
//         // Get the nodes and markets at the current level
//         let navigation: MarketNavigationResponse = match node_id {
//             Some(id) => {
//                 debug!("Getting navigation node: {}", id);
//                 match market_service.get_market_navigation_node(session, id).await {
//                     Ok(response) => {
//                         debug!(
//                             "Response received for node {}: {} nodes, {} markets",
//                             id,
//                             response.nodes.len(),
//                             response.markets.len()
//                         );
//                         response
//                     }
//                     Err(e) => {
//                         error!("Error getting node {}: {:?}", id, e);
//                         // If we hit a rate limit, return empty results instead of failing
//                         if matches!(e, AppError::RateLimitExceeded | AppError::Unexpected(_)) {
//                             info!("Rate limit or API error encountered, returning partial results");
//                             return Ok(Vec::new());
//                         }
//                         return Err(e);
//                     }
//                 }
//             }
//             None => {
//                 debug!("Getting top-level navigation nodes");
//                 match market_service.get_market_navigation(session).await {
//                     Ok(response) => {
//                         debug!(
//                             "Response received for top-level nodes: {} nodes, {} markets",
//                             response.nodes.len(),
//                             response.markets.len()
//                         );
//                         response
//                     }
//                     Err(e) => {
//                         error!("Error getting top-level nodes: {:?}", e);
//                         return Err(e);
//                     }
//                 }
//             }
//         };
//
//         let mut nodes = Vec::new();
//
//         // Process all nodes at this level
//         let nodes_to_process = navigation.nodes;
//
//         // Release the semaphore before processing child nodes
//         // This allows other requests to be processed while we wait
//         // for recursive requests to complete
//         drop(_permit);
//
//         // Process nodes sequentially with rate limiting
//         // This is important to respect the API rate limits
//         // By processing nodes sequentially, we allow the rate limiter
//         // to properly control the flow of requests
//         for node in nodes_to_process.into_iter() {
//             // Recursively get the children of this node
//             match build_market_hierarchy(market_service, session, Some(&node.id), depth + 1).await {
//                 Ok(children) => {
//                     info!("Adding node {} with {} children", node.name, children.len());
//                     nodes.push(MarketNode {
//                         id: node.id.clone(),
//                         name: node.name.clone(),
//                         children,
//                         markets: Vec::new(),
//                     });
//                 }
//                 Err(e) => {
//                     error!("Error building hierarchy for node {}: {:?}", node.id, e);
//                     // Continuar con otros nodos incluso si uno falla
//                     if depth < 7 {
//                         nodes.push(MarketNode {
//                             id: node.id.clone(),
//                             name: format!("{} (error: {})", node.name, e),
//                             children: Vec::new(),
//                             markets: Vec::new(),
//                         });
//                     }
//                 }
//             }
//         }
//
//         // Process all markets in this node
//         let markets_to_process = navigation.markets;
//         for market in markets_to_process {
//             debug!("Adding market: {}", market.instrument_name);
//             nodes.push(MarketNode {
//                 id: market.epic.clone(),
//                 name: market.instrument_name.clone(),
//                 children: Vec::new(),
//                 markets: vec![market],
//             });
//         }
//
//         Ok(nodes)
//     })
// }
//
// /// Recursively extract all markets from the hierarchy into a flat list
// pub fn extract_markets_from_hierarchy(
//     nodes: &[MarketNode],
// ) -> Vec<crate::application::models::market::MarketData> {
//     let mut all_markets = Vec::new();
//
//     for node in nodes {
//         // Add markets from this node
//         all_markets.extend(node.markets.clone());
//
//         // Recursively add markets from child nodes
//         if !node.children.is_empty() {
//             all_markets.extend(extract_markets_from_hierarchy(&node.children));
//         }
//     }
//
//     all_markets
// }
