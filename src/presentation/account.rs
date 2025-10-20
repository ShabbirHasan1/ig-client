use crate::presentation::instrument::InstrumentType;
use crate::presentation::market::MarketState;
use crate::presentation::order::{Direction, OrderType, Status, TimeInForce};
use crate::presentation::serialization::string_as_float_opt;
use lightstreamer_rs::subscription::ItemUpdate;
use pretty_simple_display::DisplaySimple;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::ops::Add;

/// Account information
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfo {
    /// List of accounts owned by the user
    pub accounts: Vec<Account>,
}

/// Details of a specific account
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    /// Unique identifier for the account
    #[serde(rename = "accountId")]
    pub account_id: String,
    /// Name of the account
    #[serde(rename = "accountName")]
    pub account_name: String,
    /// Type of the account (e.g., CFD, Spread bet)
    #[serde(rename = "accountType")]
    pub account_type: String,
    /// Balance information for the account
    pub balance: AccountBalance,
    /// Base currency of the account
    pub currency: String,
    /// Current status of the account
    pub status: String,
    /// Whether this is the preferred account
    pub preferred: bool,
}

/// Account balance information
#[derive(Debug, Clone, Deserialize)]
pub struct AccountBalance {
    /// Total balance of the account
    pub balance: f64,
    /// Deposit amount
    pub deposit: f64,
    /// Current profit or loss
    #[serde(rename = "profitLoss")]
    pub profit_loss: f64,
    /// Available funds for trading
    pub available: f64,
}

/// Metadata for activity pagination
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityMetadata {
    /// Paging information
    pub paging: Option<ActivityPaging>,
}

/// Paging information for activities
#[derive(Debug, Clone, Deserialize)]
pub struct ActivityPaging {
    /// Number of items per page
    pub size: Option<i32>,
    /// URL for the next page of results
    pub next: Option<String>,
}

#[derive(Debug, Copy, Clone, DisplaySimple, Deserialize, Serialize)]
/// Type of account activity
pub enum ActivityType {
    /// Activity related to editing stop and limit orders
    #[serde(rename = "EDIT_STOP_AND_LIMIT")]
    EditStopAndLimit,
    /// Activity related to positions
    #[serde(rename = "POSITION")]
    Position,
    /// System-generated activity
    #[serde(rename = "SYSTEM")]
    System,
    /// Activity related to working orders
    #[serde(rename = "WORKING_ORDER")]
    WorkingOrder,
}

/// Individual activity record
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct Activity {
    /// Date and time of the activity
    pub date: String,
    /// Unique identifier for the deal
    #[serde(rename = "dealId", default)]
    pub deal_id: Option<String>,
    /// Instrument EPIC identifier
    #[serde(default)]
    pub epic: Option<String>,
    /// Time period of the activity
    #[serde(default)]
    pub period: Option<String>,
    /// Client-generated reference for the deal
    #[serde(rename = "dealReference", default)]
    pub deal_reference: Option<String>,
    /// Type of activity
    #[serde(rename = "type")]
    pub activity_type: ActivityType,
    /// Status of the activity
    #[serde(default)]
    pub status: Option<Status>,
    /// Description of the activity
    #[serde(default)]
    pub description: Option<String>,
    /// Additional details about the activity
    /// This is a string when detailed=false, and an object when detailed=true
    #[serde(default)]
    pub details: Option<ActivityDetails>,
    /// Channel the activity occurred on (e.g., "WEB" or "Mobile")
    #[serde(default)]
    pub channel: Option<String>,
    /// The currency, e.g., a pound symbol
    #[serde(default)]
    pub currency: Option<String>,
    /// Price level
    #[serde(default)]
    pub level: Option<String>,
}

/// Detailed information about an activity
/// Only available when using the detailed=true parameter
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct ActivityDetails {
    /// Client-generated reference for the deal
    #[serde(rename = "dealReference", default)]
    pub deal_reference: Option<String>,
    /// List of actions associated with this activity
    #[serde(default)]
    pub actions: Vec<ActivityAction>,
    /// Name of the market
    #[serde(rename = "marketName", default)]
    pub market_name: Option<String>,
    /// Date until which the order is valid
    #[serde(rename = "goodTillDate", default)]
    pub good_till_date: Option<String>,
    /// Currency of the transaction
    #[serde(default)]
    pub currency: Option<String>,
    /// Size/quantity of the transaction
    #[serde(default)]
    pub size: Option<f64>,
    /// Direction of the transaction (BUY or SELL)
    #[serde(default)]
    pub direction: Option<Direction>,
    /// Price level
    #[serde(default)]
    pub level: Option<f64>,
    /// Stop level price
    #[serde(rename = "stopLevel", default)]
    pub stop_level: Option<f64>,
    /// Distance for the stop
    #[serde(rename = "stopDistance", default)]
    pub stop_distance: Option<f64>,
    /// Whether the stop is guaranteed
    #[serde(rename = "guaranteedStop", default)]
    pub guaranteed_stop: Option<bool>,
    /// Distance for the trailing stop
    #[serde(rename = "trailingStopDistance", default)]
    pub trailing_stop_distance: Option<f64>,
    /// Step size for the trailing stop
    #[serde(rename = "trailingStep", default)]
    pub trailing_step: Option<f64>,
    /// Limit level price
    #[serde(rename = "limitLevel", default)]
    pub limit_level: Option<f64>,
    /// Distance for the limit
    #[serde(rename = "limitDistance", default)]
    pub limit_distance: Option<f64>,
}

/// Types of actions that can be performed on an activity
#[derive(Debug, Copy, Clone, DisplaySimple, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActionType {
    /// A limit order was deleted
    LimitOrderDeleted,
    /// A limit order was filled
    LimitOrderFilled,
    /// A limit order was opened
    LimitOrderOpened,
    /// A limit order was rolled
    LimitOrderRolled,
    /// A position was closed
    PositionClosed,
    /// A position was deleted
    PositionDeleted,
    /// A position was opened
    PositionOpened,
    /// A position was partially closed
    PositionPartiallyClosed,
    /// A position was rolled
    PositionRolled,
    /// A stop/limit was amended
    StopLimitAmended,
    /// A stop order was amended
    StopOrderAmended,
    /// A stop order was deleted
    StopOrderDeleted,
    /// A stop order was filled
    StopOrderFilled,
    /// A stop order was opened
    StopOrderOpened,
    /// A stop order was rolled
    StopOrderRolled,
    /// Unknown action type
    Unknown,
    /// A working order was deleted
    WorkingOrderDeleted,
}

/// Action associated with an activity
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityAction {
    /// Type of action
    pub action_type: ActionType,
    /// Deal ID affected by this action
    pub affected_deal_id: Option<String>,
}

/// Individual position
#[derive(Debug, Clone, DisplaySimple, Serialize, Deserialize)]
pub struct Position {
    /// Details of the position
    pub position: PositionDetails,
    /// Market information for the position
    pub market: PositionMarket,
    /// Profit and loss for the position
    pub pnl: Option<f64>,
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        if self.market.epic != other.market.epic {
            panic!("Cannot add positions from different markets");
        }
        Position {
            position: self.position + other.position,
            market: self.market,
            pnl: match (self.pnl, other.pnl) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
        }
    }
}

/// Details of a position
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct PositionDetails {
    /// Size of one contract
    #[serde(rename = "contractSize")]
    pub contract_size: f64,
    /// Date and time when the position was created
    #[serde(rename = "createdDate")]
    pub created_date: String,
    /// UTC date and time when the position was created
    #[serde(rename = "createdDateUTC")]
    pub created_date_utc: String,
    /// Unique identifier for the deal
    #[serde(rename = "dealId")]
    pub deal_id: String,
    /// Client-generated reference for the deal
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
    /// Direction of the position (buy or sell)
    pub direction: Direction,
    /// Price level for take profit
    #[serde(rename = "limitLevel")]
    pub limit_level: Option<f64>,
    /// Opening price level of the position
    pub level: f64,
    /// Size/quantity of the position
    pub size: f64,
    /// Price level for stop loss
    #[serde(rename = "stopLevel")]
    pub stop_level: Option<f64>,
    /// Step size for trailing stop
    #[serde(rename = "trailingStep")]
    pub trailing_step: Option<f64>,
    /// Distance for trailing stop
    #[serde(rename = "trailingStopDistance")]
    pub trailing_stop_distance: Option<f64>,
    /// Currency of the position
    pub currency: String,
    /// Whether the position has controlled risk
    #[serde(rename = "controlledRisk")]
    pub controlled_risk: bool,
    /// Premium paid for limited risk
    #[serde(rename = "limitedRiskPremium")]
    pub limited_risk_premium: Option<f64>,
}

impl Add for PositionDetails {
    type Output = PositionDetails;

    fn add(self, other: PositionDetails) -> PositionDetails {
        let (contract_size, size) = if self.direction != other.direction {
            (
                (self.contract_size - other.contract_size).abs(),
                (self.size - other.size).abs(),
            )
        } else {
            (
                self.contract_size + other.contract_size,
                self.size + other.size,
            )
        };

        PositionDetails {
            contract_size,
            created_date: self.created_date,
            created_date_utc: self.created_date_utc,
            deal_id: self.deal_id,
            deal_reference: self.deal_reference,
            direction: self.direction,
            limit_level: other.limit_level.or(self.limit_level),
            level: (self.level + other.level) / 2.0, // Average level
            size,
            stop_level: other.stop_level.or(self.stop_level),
            trailing_step: other.trailing_step.or(self.trailing_step),
            trailing_stop_distance: other.trailing_stop_distance.or(self.trailing_stop_distance),
            currency: self.currency.clone(),
            controlled_risk: self.controlled_risk || other.controlled_risk,
            limited_risk_premium: other.limited_risk_premium.or(self.limited_risk_premium),
        }
    }
}

/// Market information for a position
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct PositionMarket {
    /// Human-readable name of the instrument
    #[serde(rename = "instrumentName")]
    pub instrument_name: String,
    /// Expiry date of the instrument
    pub expiry: String,
    /// Unique identifier for the market
    pub epic: String,
    /// Type of the instrument
    #[serde(rename = "instrumentType")]
    pub instrument_type: String,
    /// Size of one lot
    #[serde(rename = "lotSize")]
    pub lot_size: f64,
    /// Highest price of the current trading session
    pub high: Option<f64>,
    /// Lowest price of the current trading session
    pub low: Option<f64>,
    /// Percentage change in price since previous close
    #[serde(rename = "percentageChange")]
    pub percentage_change: f64,
    /// Net change in price since previous close
    #[serde(rename = "netChange")]
    pub net_change: f64,
    /// Current bid price
    pub bid: Option<f64>,
    /// Current offer/ask price
    pub offer: Option<f64>,
    /// Time of the last price update
    #[serde(rename = "updateTime")]
    pub update_time: String,
    /// UTC time of the last price update
    #[serde(rename = "updateTimeUTC")]
    pub update_time_utc: String,
    /// Delay time in milliseconds for market data
    #[serde(rename = "delayTime")]
    pub delay_time: i64,
    /// Whether streaming prices are available for this market
    #[serde(rename = "streamingPricesAvailable")]
    pub streaming_prices_available: bool,
    /// Current status of the market (e.g., "OPEN", "CLOSED")
    #[serde(rename = "marketStatus")]
    pub market_status: String,
    /// Factor for scaling prices
    #[serde(rename = "scalingFactor")]
    pub scaling_factor: i64,
}

/// Working order
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct WorkingOrder {
    /// Details of the working order
    #[serde(rename = "workingOrderData")]
    pub working_order_data: WorkingOrderData,
    /// Market information for the working order
    #[serde(rename = "marketData")]
    pub market_data: AccountMarketData,
}

/// Details of a working order
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct WorkingOrderData {
    /// Unique identifier for the deal
    #[serde(rename = "dealId")]
    pub deal_id: String,
    /// Direction of the order (buy or sell)
    pub direction: Direction,
    /// Instrument EPIC identifier
    pub epic: String,
    /// Size/quantity of the order
    #[serde(rename = "orderSize")]
    pub order_size: f64,
    /// Price level for the order
    #[serde(rename = "orderLevel")]
    pub order_level: f64,
    /// Time in force for the order
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Expiry date for GTD orders
    #[serde(rename = "goodTillDate")]
    pub good_till_date: Option<String>,
    /// ISO formatted expiry date for GTD orders
    #[serde(rename = "goodTillDateISO")]
    pub good_till_date_iso: Option<String>,
    /// Date and time when the order was created
    #[serde(rename = "createdDate")]
    pub created_date: String,
    /// UTC date and time when the order was created
    #[serde(rename = "createdDateUTC")]
    pub created_date_utc: String,
    /// Whether the order has a guaranteed stop
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool,
    /// Type of the order
    #[serde(rename = "orderType")]
    pub order_type: OrderType,
    /// Distance for stop loss
    #[serde(rename = "stopDistance")]
    pub stop_distance: Option<f64>,
    /// Distance for take profit
    #[serde(rename = "limitDistance")]
    pub limit_distance: Option<f64>,
    /// Currency code for the order
    #[serde(rename = "currencyCode")]
    pub currency_code: String,
    /// Whether direct market access is enabled
    pub dma: bool,
    /// Premium for limited risk
    #[serde(rename = "limitedRiskPremium")]
    pub limited_risk_premium: Option<f64>,
    /// Price level for take profit
    #[serde(rename = "limitLevel", default)]
    pub limit_level: Option<f64>,
    /// Price level for stop loss
    #[serde(rename = "stopLevel", default)]
    pub stop_level: Option<f64>,
    /// Client-generated reference for the deal
    #[serde(rename = "dealReference", default)]
    pub deal_reference: Option<String>,
}

/// Market data for a working order
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct AccountMarketData {
    /// Human-readable name of the instrument
    #[serde(rename = "instrumentName")]
    pub instrument_name: String,
    /// Exchange identifier
    #[serde(rename = "exchangeId")]
    pub exchange_id: String,
    /// Expiry date of the instrument
    pub expiry: String,
    /// Current status of the market
    #[serde(rename = "marketStatus")]
    pub market_status: MarketState,
    /// Unique identifier for the market
    pub epic: String,
    /// Type of the instrument
    #[serde(rename = "instrumentType")]
    pub instrument_type: InstrumentType,
    /// Size of one lot
    #[serde(rename = "lotSize")]
    pub lot_size: f64,
    /// Highest price of the current trading session
    pub high: Option<f64>,
    /// Lowest price of the current trading session
    pub low: Option<f64>,
    /// Percentage change in price since previous close
    #[serde(rename = "percentageChange")]
    pub percentage_change: f64,
    /// Net change in price since previous close
    #[serde(rename = "netChange")]
    pub net_change: f64,
    /// Current bid price
    pub bid: Option<f64>,
    /// Current offer/ask price
    pub offer: Option<f64>,
    /// Time of the last price update
    #[serde(rename = "updateTime")]
    pub update_time: String,
    /// UTC time of the last price update
    #[serde(rename = "updateTimeUTC")]
    pub update_time_utc: String,
    /// Delay time in milliseconds for market data
    #[serde(rename = "delayTime")]
    pub delay_time: i64,
    /// Whether streaming prices are available for this market
    #[serde(rename = "streamingPricesAvailable")]
    pub streaming_prices_available: bool,
    /// Factor for scaling prices
    #[serde(rename = "scalingFactor")]
    pub scaling_factor: i64,
}

/// Transaction metadata
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct TransactionMetadata {
    /// Pagination information
    #[serde(rename = "pageData")]
    pub page_data: PageData,
    /// Total number of transactions
    pub size: i32,
}

/// Pagination information
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct PageData {
    /// Current page number
    #[serde(rename = "pageNumber")]
    pub page_number: i32,
    /// Number of items per page
    #[serde(rename = "pageSize")]
    pub page_size: i32,
    /// Total number of pages
    #[serde(rename = "totalPages")]
    pub total_pages: i32,
}

/// Individual transaction
#[derive(Debug, Clone, DisplaySimple, Deserialize, Serialize)]
pub struct AccountTransaction {
    /// Date and time of the transaction
    pub date: String,
    /// UTC date and time of the transaction
    #[serde(rename = "dateUtc")]
    pub date_utc: String,
    /// Represents the date and time in UTC when an event or entity was opened or initiated.
    #[serde(rename = "openDateUtc")]
    pub open_date_utc: String,
    /// Name of the instrument
    #[serde(rename = "instrumentName")]
    pub instrument_name: String,
    /// Time period of the transaction
    pub period: String,
    /// Profit or loss amount
    #[serde(rename = "profitAndLoss")]
    pub profit_and_loss: String,
    /// Type of transaction
    #[serde(rename = "transactionType")]
    pub transaction_type: String,
    /// Reference identifier for the transaction
    pub reference: String,
    /// Opening price level
    #[serde(rename = "openLevel")]
    pub open_level: String,
    /// Closing price level
    #[serde(rename = "closeLevel")]
    pub close_level: String,
    /// Size/quantity of the transaction
    pub size: String,
    /// Currency of the transaction
    pub currency: String,
    /// Whether this is a cash transaction
    #[serde(rename = "cashTransaction")]
    pub cash_transaction: bool,
}

/// Representation of account data received from the IG Markets streaming API
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountData {
    /// Name of the item this data belongs to
    item_name: String,
    /// Position of the item in the subscription
    item_pos: i32,
    /// All account fields
    fields: AccountFields,
    /// Fields that have changed in this update
    changed_fields: AccountFields,
    /// Whether this is a snapshot or an update
    is_snapshot: bool,
}

/// Fields containing account financial information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountFields {
    #[serde(rename = "PNL")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pnl: Option<f64>,

    #[serde(rename = "DEPOSIT")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    deposit: Option<f64>,

    #[serde(rename = "AVAILABLE_CASH")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    available_cash: Option<f64>,

    #[serde(rename = "PNL_LR")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pnl_lr: Option<f64>,

    #[serde(rename = "PNL_NLR")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pnl_nlr: Option<f64>,

    #[serde(rename = "FUNDS")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    funds: Option<f64>,

    #[serde(rename = "MARGIN")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    margin: Option<f64>,

    #[serde(rename = "MARGIN_LR")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    margin_lr: Option<f64>,

    #[serde(rename = "MARGIN_NLR")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    margin_nlr: Option<f64>,

    #[serde(rename = "AVAILABLE_TO_DEAL")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    available_to_deal: Option<f64>,

    #[serde(rename = "EQUITY")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    equity: Option<f64>,

    #[serde(rename = "EQUITY_USED")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    equity_used: Option<f64>,
}

impl AccountData {
    /// Converts an ItemUpdate from the Lightstreamer API to an AccountData object
    ///
    /// # Arguments
    /// * `item_update` - The ItemUpdate received from the Lightstreamer API
    ///
    /// # Returns
    /// * `Result<Self, String>` - The converted AccountData or an error message
    pub fn from_item_update(item_update: &ItemUpdate) -> Result<Self, String> {
        // Extract the item_name, defaulting to an empty string if None
        let item_name = item_update.item_name.clone().unwrap_or_default();

        // Convert item_pos from usize to i32
        let item_pos = item_update.item_pos as i32;

        // Extract is_snapshot
        let is_snapshot = item_update.is_snapshot;

        // Convert fields
        let fields = Self::create_account_fields(&item_update.fields)?;

        // Convert changed_fields by first creating a HashMap<String, Option<String>>
        let mut changed_fields_map: HashMap<String, Option<String>> = HashMap::new();
        for (key, value) in &item_update.changed_fields {
            changed_fields_map.insert(key.clone(), Some(value.clone()));
        }
        let changed_fields = Self::create_account_fields(&changed_fields_map)?;

        Ok(AccountData {
            item_name,
            item_pos,
            fields,
            changed_fields,
            is_snapshot,
        })
    }

    /// Helper method to create AccountFields from a HashMap of field values
    ///
    /// # Arguments
    /// * `fields_map` - HashMap containing field names and their string values
    ///
    /// # Returns
    /// * `Result<AccountFields, String>` - The parsed AccountFields or an error message
    fn create_account_fields(
        fields_map: &HashMap<String, Option<String>>,
    ) -> Result<AccountFields, String> {
        // Helper function to safely get a field value
        let get_field = |key: &str| -> Option<String> { fields_map.get(key).cloned().flatten() };

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

        Ok(AccountFields {
            pnl: parse_float("PNL")?,
            deposit: parse_float("DEPOSIT")?,
            available_cash: parse_float("AVAILABLE_CASH")?,
            pnl_lr: parse_float("PNL_LR")?,
            pnl_nlr: parse_float("PNL_NLR")?,
            funds: parse_float("FUNDS")?,
            margin: parse_float("MARGIN")?,
            margin_lr: parse_float("MARGIN_LR")?,
            margin_nlr: parse_float("MARGIN_NLR")?,
            available_to_deal: parse_float("AVAILABLE_TO_DEAL")?,
            equity: parse_float("EQUITY")?,
            equity_used: parse_float("EQUITY_USED")?,
        })
    }
}

impl fmt::Display for AccountData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{json}")
    }
}

impl From<&ItemUpdate> for AccountData {
    fn from(item_update: &ItemUpdate) -> Self {
        Self::from_item_update(item_update).unwrap_or_else(|_| AccountData::default())
    }
}
