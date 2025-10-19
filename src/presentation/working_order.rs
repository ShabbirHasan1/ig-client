use crate::presentation::order::{Direction, OrderType, TimeInForce};
use serde::{Deserialize, Serialize};

/// Model for creating a new working order
#[derive(Debug, Clone, Serialize)]
pub struct CreateWorkingOrderRequest {
    /// Instrument EPIC identifier
    pub epic: String,
    /// Order direction (buy or sell)
    pub direction: Direction,
    /// Order size/quantity
    pub size: f64,
    /// Price level for the order
    pub level: f64,
    /// Type of working order (LIMIT or STOP)
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order duration (how long the order remains valid)
    #[serde(rename = "timeInForce")]
    pub time_in_force: TimeInForce,
    /// Whether to use a guaranteed stop
    #[serde(rename = "guaranteedStop")]
    pub guaranteed_stop: bool,
    /// Price level for stop loss
    #[serde(rename = "stopLevel", skip_serializing_if = "Option::is_none")]
    pub stop_level: Option<f64>,
    /// Distance for stop loss
    #[serde(rename = "stopDistance", skip_serializing_if = "Option::is_none")]
    pub stop_distance: Option<f64>,
    /// Price level for take profit
    #[serde(rename = "limitLevel", skip_serializing_if = "Option::is_none")]
    pub limit_level: Option<f64>,
    /// Distance for take profit
    #[serde(rename = "limitDistance", skip_serializing_if = "Option::is_none")]
    pub limit_distance: Option<f64>,
    /// Expiry date for GTD orders
    #[serde(rename = "goodTillDate", skip_serializing_if = "Option::is_none")]
    pub good_till_date: Option<String>,
    /// Client-generated reference for the deal
    #[serde(rename = "dealReference", skip_serializing_if = "Option::is_none")]
    pub deal_reference: Option<String>,
    /// Currency code for the order (e.g., "USD", "EUR")
    #[serde(rename = "currencyCode", skip_serializing_if = "Option::is_none")]
    pub currency_code: Option<String>,
    /// Expiry date for the order (required by the API)
    #[serde(rename = "expiry")]
    pub expiry: String,
}

impl CreateWorkingOrderRequest {
    /// Creates a new limit working order
    pub fn limit(epic: String, direction: Direction, size: f64, level: f64) -> Self {
        Self {
            epic,
            direction,
            size,
            level,
            order_type: OrderType::Limit,
            time_in_force: TimeInForce::GoodTillCancelled,
            guaranteed_stop: false,
            stop_level: None,
            stop_distance: None,
            limit_level: None,
            limit_distance: None,
            good_till_date: None,
            deal_reference: None,
            currency_code: None,
            expiry: "DFB".to_string(), // Default expiry for Daily Forward Bet
        }
    }

    /// Creates a new stop working order
    pub fn stop(epic: String, direction: Direction, size: f64, level: f64) -> Self {
        Self {
            epic,
            direction,
            size,
            level,
            order_type: OrderType::Stop,
            time_in_force: TimeInForce::GoodTillCancelled,
            guaranteed_stop: false,
            stop_level: None,
            stop_distance: None,
            limit_level: None,
            limit_distance: None,
            good_till_date: None,
            deal_reference: None,
            currency_code: None,
            expiry: "DFB".to_string(), // Default expiry for Daily Forward Bet
        }
    }

    /// Adds a stop loss to the working order
    pub fn with_stop_loss(mut self, stop_level: f64) -> Self {
        self.stop_level = Some(stop_level);
        self
    }

    /// Adds a take profit to the working order
    pub fn with_take_profit(mut self, limit_level: f64) -> Self {
        self.limit_level = Some(limit_level);
        self
    }

    /// Adds a reference to the working order
    pub fn with_reference(mut self, reference: String) -> Self {
        self.deal_reference = Some(reference);
        self
    }

    /// Sets the order to expire at a specific date
    pub fn expires_at(mut self, date: String) -> Self {
        self.time_in_force = TimeInForce::GoodTillDate;
        self.good_till_date = Some(date);
        self
    }

    /// Sets the expiry for the order
    pub fn with_expiry(mut self, expiry: String) -> Self {
        self.expiry = expiry;
        self
    }
}

/// Response to working order creation
#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkingOrderResponse {
    /// Client-generated reference for the deal
    #[serde(rename = "dealReference")]
    pub deal_reference: String,
}
