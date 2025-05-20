/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/5/25
******************************************************************************/
use super::order::{Direction, OrderType, Status, TimeInForce};
use crate::application::models::market::InstrumentType;
use crate::impl_json_display;
use crate::presentation::MarketState;
use serde::{Deserialize, Serialize};

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

/// Account activity
#[derive(Debug, Clone, Deserialize)]
pub struct AccountActivity {
    /// List of activities on the account
    pub activities: Vec<Activity>,
    /// Metadata about pagination
    pub metadata: Option<ActivityMetadata>,
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

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum ActivityType {
    #[serde(rename = "EDIT_STOP_AND_LIMIT")]
    EditStopAndLimit,
    #[serde(rename = "POSITION")]
    Position,
    #[serde(rename = "SYSTEM")]
    System,
    #[serde(rename = "WORKING_ORDER")]
    WorkingOrder,
}

/// Individual activity record
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum ActionType {
    /// A limit order was deleted
    #[serde(rename = "LIMIT_ORDER_DELETED")]
    LimitOrderDeleted,
    /// A limit order was filled
    #[serde(rename = "LIMIT_ORDER_FILLED")]
    LimitOrderFilled,
    /// A limit order was opened
    #[serde(rename = "LIMIT_ORDER_OPENED")]
    LimitOrderOpened,
    /// A limit order was rolled
    #[serde(rename = "LIMIT_ORDER_ROLLED")]
    LimitOrderRolled,
    /// A position was closed
    #[serde(rename = "POSITION_CLOSED")]
    PositionClosed,
    /// A position was deleted
    #[serde(rename = "POSITION_DELETED")]
    PositionDeleted,
    /// A position was opened
    #[serde(rename = "POSITION_OPENED")]
    PositionOpened,
    /// A position was partially closed
    #[serde(rename = "POSITION_PARTIALLY_CLOSED")]
    PositionPartiallyClosed,
    /// A position was rolled
    #[serde(rename = "POSITION_ROLLED")]
    PositionRolled,
    /// A stop/limit was amended
    #[serde(rename = "STOP_LIMIT_AMENDED")]
    StopLimitAmended,
    /// A stop order was amended
    #[serde(rename = "STOP_ORDER_AMENDED")]
    StopOrderAmended,
    /// A stop order was deleted
    #[serde(rename = "STOP_ORDER_DELETED")]
    StopOrderDeleted,
    /// A stop order was filled
    #[serde(rename = "STOP_ORDER_FILLED")]
    StopOrderFilled,
    /// A stop order was opened
    #[serde(rename = "STOP_ORDER_OPENED")]
    StopOrderOpened,
    /// A stop order was rolled
    #[serde(rename = "STOP_ORDER_ROLLED")]
    StopOrderRolled,
    /// Unknown action type
    #[serde(rename = "UNKNOWN")]
    Unknown,
    /// A working order was deleted
    #[serde(rename = "WORKING_ORDER_DELETED")]
    WorkingOrderDeleted,
}

impl_json_display!(ActionType);

/// Action associated with an activity
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityAction {
    /// Type of action
    #[serde(rename = "actionType")]
    pub action_type: ActionType,
    /// Deal ID affected by this action
    #[serde(rename = "affectedDealId", default)]
    pub affected_deal_id: Option<String>,
}

/// Open positions
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Positions {
    /// List of open positions
    pub positions: Vec<Position>,
}

/// Individual position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Details of the position
    pub position: PositionDetails,
    /// Market information for the position
    pub market: PositionMarket,
    /// Profit and loss for the position
    pub pnl: Option<f64>,
}

/// Details of a position
#[derive(Debug, Clone, Deserialize, Serialize)]
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

/// Market information for a position
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub high: f64,
    /// Lowest price of the current trading session
    pub low: f64,
    /// Percentage change in price since previous close
    #[serde(rename = "percentageChange")]
    pub percentage_change: f64,
    /// Net change in price since previous close
    #[serde(rename = "netChange")]
    pub net_change: f64,
    /// Current bid price
    pub bid: f64,
    /// Current offer/ask price
    pub offer: f64,
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

/// Working orders
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkingOrders {
    /// List of pending working orders
    #[serde(rename = "workingOrders")]
    pub working_orders: Vec<WorkingOrder>,
}

/// Working order
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkingOrder {
    /// Details of the working order
    #[serde(rename = "workingOrderData")]
    pub working_order_data: WorkingOrderData,
    /// Market information for the working order
    #[serde(rename = "marketData")]
    pub market_data: MarketData,
}

/// Details of a working order
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketData {
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
    pub high: f64,
    /// Lowest price of the current trading session
    pub low: f64,
    /// Percentage change in price since previous close
    #[serde(rename = "percentageChange")]
    pub percentage_change: f64,
    /// Net change in price since previous close
    #[serde(rename = "netChange")]
    pub net_change: f64,
    /// Current bid price
    pub bid: f64,
    /// Current offer/ask price
    pub offer: f64,
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

/// Transaction history
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionHistory {
    /// List of account transactions
    pub transactions: Vec<AccountTransaction>,
    /// Metadata about the transaction list
    pub metadata: TransactionMetadata,
}

/// Transaction metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionMetadata {
    /// Pagination information
    #[serde(rename = "pageData")]
    pub page_data: PageData,
    /// Total number of transactions
    pub size: i32,
}

/// Pagination information
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountTransaction {
    /// Date and time of the transaction
    pub date: String,
    /// UTC date and time of the transaction
    #[serde(rename = "dateUtc")]
    pub date_utc: String,
    #[serde(rename = "openDateUtc")]
    pub(crate) open_date_utc: String,
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

impl_json_display!(AccountTransaction);
