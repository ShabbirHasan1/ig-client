use crate::presentation::order::{Direction, OrderType, Status, TimeInForce};
use crate::presentation::serialization::{option_string_empty_as_none, string_as_float_opt};
use lightstreamer_rs::subscription::ItemUpdate;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fmt;

/// Main structure for trade data received from the IG Markets API
/// Contains information about trades, positions and working orders
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeData {
    /// Name of the item (usually the trade ID)
    pub item_name: String,
    /// Position of the item in the subscription
    pub item_pos: i32,
    /// All trade fields for this item
    pub fields: TradeFields,
    /// Fields that have changed in this update
    pub changed_fields: TradeFields,
    /// Whether this is a snapshot or an update
    pub is_snapshot: bool,
}

/// Main fields for a trade update, containing core trade data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeFields {
    /// Optional confirmation details for the trade.
    #[serde(rename = "CONFIRMS")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub confirms: Option<String>,
    /// Optional open position update details.
    #[serde(rename = "OPU")]
    #[serde(default)]
    pub opu: Option<OpenPositionUpdate>,
    /// Optional working order update details.
    #[serde(rename = "WOU")]
    #[serde(default)]
    pub wou: Option<WorkingOrderUpdate>,
}

/// Structure representing details of an open position update.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenPositionUpdate {
    /// Unique deal reference for the open position.
    #[serde(rename = "dealReference")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub deal_reference: Option<String>,
    /// Unique deal identifier for the position.
    #[serde(rename = "dealId")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub deal_id: Option<String>,
    /// Direction of the trade position (buy or sell).
    #[serde(default)]
    pub direction: Option<Direction>,
    /// Epic identifier for the instrument.
    #[serde(default)]
    pub epic: Option<String>,
    /// Status of the position.
    #[serde(default)]
    pub status: Option<Status>,
    /// Deal status of the position.
    #[serde(rename = "dealStatus")]
    #[serde(default)]
    pub deal_status: Option<Status>,
    /// Price level of the position.
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub level: Option<f64>,
    /// Position size.
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub size: Option<f64>,
    /// Currency of the position.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub currency: Option<String>,
    /// Timestamp of the position update.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Channel through which the update was received.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub channel: Option<String>,
    /// Expiry date of the position, if applicable.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub expiry: Option<String>,
    /// Original deal identifier for the position.
    #[serde(rename = "dealIdOrigin")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub deal_id_origin: Option<String>,
}

/// Structure representing details of a working order update.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingOrderUpdate {
    /// Unique deal reference for the working order.
    #[serde(rename = "dealReference")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub deal_reference: Option<String>,
    /// Unique deal identifier for the working order.
    #[serde(rename = "dealId")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub deal_id: Option<String>,
    /// Direction of the working order (buy or sell).
    #[serde(default)]
    pub direction: Option<Direction>,
    /// Epic identifier for the working order instrument.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub epic: Option<String>,
    /// Status of the working order.
    #[serde(default)]
    pub status: Option<Status>,
    /// Deal status of the working order.
    #[serde(rename = "dealStatus")]
    #[serde(default)]
    pub deal_status: Option<Status>,
    /// Price level at which the working order is set.
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub level: Option<f64>,
    /// Working order size.
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub size: Option<f64>,
    /// Currency of the working order.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub currency: Option<String>,
    /// Timestamp of the working order update.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Channel through which the working order update was received.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub channel: Option<String>,
    /// Expiry date of the working order.
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub expiry: Option<String>,
    /// Stop distance for guaranteed stop orders.
    #[serde(rename = "stopDistance")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub stop_distance: Option<f64>,
    /// Limit distance for guaranteed stop orders.
    #[serde(rename = "limitDistance")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    pub limit_distance: Option<f64>,
    /// Whether the stop is guaranteed.
    #[serde(rename = "guaranteedStop")]
    #[serde(default)]
    pub guaranteed_stop: Option<bool>,
    /// Type of the order (e.g., market, limit).
    #[serde(rename = "orderType")]
    #[serde(default)]
    pub order_type: Option<OrderType>,
    /// Time in force for the order.
    #[serde(rename = "timeInForce")]
    #[serde(default)]
    pub time_in_force: Option<TimeInForce>,
    /// Good till date for the working order.
    #[serde(rename = "goodTillDate")]
    #[serde(with = "option_string_empty_as_none")]
    #[serde(default)]
    pub good_till_date: Option<String>,
}

impl TradeData {
    /// Converts a Lightstreamer ItemUpdate to a TradeData object
    ///
    /// # Arguments
    ///
    /// * `item_update` - The ItemUpdate from Lightstreamer containing trade data
    ///
    /// # Returns
    ///
    /// A Result containing either the parsed TradeData or an error message
    pub fn from_item_update(item_update: &ItemUpdate) -> Result<Self, String> {
        // Extract the item_name, defaulting to an empty string if None
        let item_name = item_update.item_name.clone().unwrap_or_default();

        // Convert item_pos from usize to i32
        let item_pos = item_update.item_pos as i32;

        // Extract is_snapshot
        let is_snapshot = item_update.is_snapshot;

        // Convert fields
        let fields = Self::create_trade_fields(&item_update.fields)?;

        // Convert changed_fields by first creating a HashMap<String, Option<String>>
        let mut changed_fields_map: HashMap<String, Option<String>> = HashMap::new();
        for (key, value) in &item_update.changed_fields {
            changed_fields_map.insert(key.clone(), Some(value.clone()));
        }
        let changed_fields = Self::create_trade_fields(&changed_fields_map)?;

        Ok(TradeData {
            item_name,
            item_pos,
            fields,
            changed_fields,
            is_snapshot,
        })
    }

    // Helper method to create TradeFields from a HashMap
    fn create_trade_fields(
        fields_map: &HashMap<String, Option<String>>,
    ) -> Result<TradeFields, String> {
        // Helper function to safely get a field value
        let get_field = |key: &str| -> Option<String> {
            let field = fields_map.get(key).cloned().flatten();
            match field {
                Some(ref s) if s.is_empty() => None,
                _ => field,
            }
        };

        // Parse CONFIRMS
        let confirms = get_field("CONFIRMS");

        // Parse OPU
        let opu_str = get_field("OPU");
        let opu = if let Some(opu_json) = opu_str {
            if !opu_json.is_empty() {
                match serde_json::from_str::<OpenPositionUpdate>(&opu_json) {
                    Ok(parsed_opu) => Some(parsed_opu),
                    Err(e) => return Err(format!("Failed to parse OPU JSON: {e}")),
                }
            } else {
                None
            }
        } else {
            None
        };
        // Parse WOU
        let wou_str = get_field("WOU");
        let wou = if let Some(wou_json) = wou_str {
            if !wou_json.is_empty() {
                match serde_json::from_str::<WorkingOrderUpdate>(&wou_json) {
                    Ok(parsed_wou) => Some(parsed_wou),
                    Err(e) => return Err(format!("Failed to parse WOU JSON: {e}")),
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(TradeFields { confirms, opu, wou })
    }
}

impl fmt::Display for TradeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{json}")
    }
}

impl From<&ItemUpdate> for TradeData {
    fn from(item_update: &ItemUpdate) -> Self {
        Self::from_item_update(item_update).unwrap_or_default()
    }
}
