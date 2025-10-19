use crate::presentation::serialization::string_as_float_opt;
use lightstreamer_rs::subscription::ItemUpdate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Time scale for chart data aggregation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ChartScale {
    /// Second-level aggregation
    #[serde(rename = "SECOND")]
    Second,
    /// One-minute aggregation
    #[serde(rename = "1MINUTE")]
    OneMinute,
    /// Five-minute aggregation
    #[serde(rename = "5MINUTE")]
    FiveMinute,
    /// Hourly aggregation
    #[serde(rename = "HOUR")]
    Hour,
    /// Tick-by-tick data (no aggregation)
    #[serde(rename = "TICK")]
    #[default]
    Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Chart data structure that represents price chart information
/// Contains both tick and candle data depending on the chart scale
pub struct ChartData {
    item_name: String,
    item_pos: i32,
    #[serde(default)]
    scale: ChartScale, // Derived from the item name or the {scale} field
    fields: ChartFields,
    changed_fields: ChartFields,
    is_snapshot: bool,
}

/// Chart field data containing price, volume, and timestamp information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChartFields {
    // Common fields for both chart types
    #[serde(rename = "LTV")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    last_traded_volume: Option<f64>,

    #[serde(rename = "TTV")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    incremental_trading_volume: Option<f64>,

    #[serde(rename = "UTM")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    update_time: Option<f64>,

    #[serde(rename = "DAY_OPEN_MID")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    day_open_mid: Option<f64>,

    #[serde(rename = "DAY_NET_CHG_MID")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    day_net_change_mid: Option<f64>,

    #[serde(rename = "DAY_PERC_CHG_MID")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    day_percentage_change_mid: Option<f64>,

    #[serde(rename = "DAY_HIGH")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    day_high: Option<f64>,

    #[serde(rename = "DAY_LOW")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    day_low: Option<f64>,

    // Fields specific to TICK
    #[serde(rename = "BID")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    bid: Option<f64>,

    #[serde(rename = "OFR")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    offer: Option<f64>,

    #[serde(rename = "LTP")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    last_traded_price: Option<f64>,

    // Fields specific to CANDLE
    #[serde(rename = "OFR_OPEN")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    offer_open: Option<f64>,

    #[serde(rename = "OFR_HIGH")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    offer_high: Option<f64>,

    #[serde(rename = "OFR_LOW")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    offer_low: Option<f64>,

    #[serde(rename = "OFR_CLOSE")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    offer_close: Option<f64>,

    #[serde(rename = "BID_OPEN")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    bid_open: Option<f64>,

    #[serde(rename = "BID_HIGH")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    bid_high: Option<f64>,

    #[serde(rename = "BID_LOW")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    bid_low: Option<f64>,

    #[serde(rename = "BID_CLOSE")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    bid_close: Option<f64>,

    #[serde(rename = "LTP_OPEN")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    ltp_open: Option<f64>,

    #[serde(rename = "LTP_HIGH")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    ltp_high: Option<f64>,

    #[serde(rename = "LTP_LOW")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    ltp_low: Option<f64>,

    #[serde(rename = "LTP_CLOSE")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    ltp_close: Option<f64>,

    #[serde(rename = "CONS_END")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    candle_end: Option<f64>,

    #[serde(rename = "CONS_TICK_COUNT")]
    #[serde(with = "string_as_float_opt")]
    #[serde(default)]
    candle_tick_count: Option<f64>,
}

impl ChartData {
    /// Converts a Lightstreamer ItemUpdate to a ChartData object
    ///
    /// # Arguments
    ///
    /// * `item_update` - The ItemUpdate from Lightstreamer containing chart data
    ///
    /// # Returns
    ///
    /// A Result containing either the parsed ChartData or an error message
    pub fn from_item_update(item_update: &ItemUpdate) -> Result<Self, String> {
        // Extract the item_name, defaulting to an empty string if None
        let item_name = item_update.item_name.clone().unwrap_or_default();

        // Determine the chart scale from the item name
        let scale = if let Some(item_name) = &item_update.item_name {
            if item_name.ends_with(":TICK") {
                ChartScale::Tick
            } else if item_name.ends_with(":SECOND") {
                ChartScale::Second
            } else if item_name.ends_with(":1MINUTE") {
                ChartScale::OneMinute
            } else if item_name.ends_with(":5MINUTE") {
                ChartScale::FiveMinute
            } else if item_name.ends_with(":HOUR") {
                ChartScale::Hour
            } else {
                // Try to determine the scale from a {scale} field if it exists
                match item_update.fields.get("{scale}").and_then(|s| s.as_ref()) {
                    Some(s) if s == "SECOND" => ChartScale::Second,
                    Some(s) if s == "1MINUTE" => ChartScale::OneMinute,
                    Some(s) if s == "5MINUTE" => ChartScale::FiveMinute,
                    Some(s) if s == "HOUR" => ChartScale::Hour,
                    _ => ChartScale::Tick, // Default
                }
            }
        } else {
            ChartScale::default()
        };

        // Convert item_pos from usize to i32
        let item_pos = item_update.item_pos as i32;

        // Extract is_snapshot
        let is_snapshot = item_update.is_snapshot;

        // Convert fields
        let fields = Self::create_chart_fields(&item_update.fields)?;

        // Convert changed_fields by first creating a HashMap<String, Option<String>>
        let mut changed_fields_map: HashMap<String, Option<String>> = HashMap::new();
        for (key, value) in &item_update.changed_fields {
            changed_fields_map.insert(key.clone(), Some(value.clone()));
        }
        let changed_fields = Self::create_chart_fields(&changed_fields_map)?;

        Ok(ChartData {
            item_name,
            item_pos,
            scale,
            fields,
            changed_fields,
            is_snapshot,
        })
    }

    // Helper method to create ChartFields from a HashMap
    fn create_chart_fields(
        fields_map: &HashMap<String, Option<String>>,
    ) -> Result<ChartFields, String> {
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

        Ok(ChartFields {
            // Common fields
            last_traded_volume: parse_float("LTV")?,
            incremental_trading_volume: parse_float("TTV")?,
            update_time: parse_float("UTM")?,
            day_open_mid: parse_float("DAY_OPEN_MID")?,
            day_net_change_mid: parse_float("DAY_NET_CHG_MID")?,
            day_percentage_change_mid: parse_float("DAY_PERC_CHG_MID")?,
            day_high: parse_float("DAY_HIGH")?,
            day_low: parse_float("DAY_LOW")?,

            // Fields specific to TICK
            bid: parse_float("BID")?,
            offer: parse_float("OFR")?,
            last_traded_price: parse_float("LTP")?,

            // Fields specific to CANDLE
            offer_open: parse_float("OFR_OPEN")?,
            offer_high: parse_float("OFR_HIGH")?,
            offer_low: parse_float("OFR_LOW")?,
            offer_close: parse_float("OFR_CLOSE")?,
            bid_open: parse_float("BID_OPEN")?,
            bid_high: parse_float("BID_HIGH")?,
            bid_low: parse_float("BID_LOW")?,
            bid_close: parse_float("BID_CLOSE")?,
            ltp_open: parse_float("LTP_OPEN")?,
            ltp_high: parse_float("LTP_HIGH")?,
            ltp_low: parse_float("LTP_LOW")?,
            ltp_close: parse_float("LTP_CLOSE")?,
            candle_end: parse_float("CONS_END")?,
            candle_tick_count: parse_float("CONS_TICK_COUNT")?,
        })
    }

    /// Checks if these chart data are of type TICK
    pub fn is_tick(&self) -> bool {
        matches!(self.scale, ChartScale::Tick)
    }

    /// Checks if these chart data are of type CANDLE (any time scale)
    pub fn is_candle(&self) -> bool {
        !self.is_tick()
    }

    /// Gets the time scale of the data
    pub fn get_scale(&self) -> &ChartScale {
        &self.scale
    }
}

impl fmt::Display for ChartData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{json}")
    }
}

impl From<&ItemUpdate> for ChartData {
    fn from(item_update: &ItemUpdate) -> Self {
        Self::from_item_update(item_update).unwrap_or_default()
    }
}
