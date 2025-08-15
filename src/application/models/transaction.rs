use crate::application::models::account::AccountTransaction;
use crate::impl_json_display;
use crate::utils::parsing::{ParsedOptionInfo, parse_instrument_name};
use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Represents a processed transaction from IG Markets with parsed fields
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreTransaction {
    /// Date and time when the transaction was executed
    pub deal_date: DateTime<Utc>,
    /// Underlying asset or instrument (e.g., "GOLD", "US500")
    pub underlying: Option<String>,
    /// Strike price for options
    pub strike: Option<f64>,
    /// Type of option ("CALL" or "PUT")
    pub option_type: Option<String>,
    /// Expiration date for options
    pub expiry: Option<NaiveDate>,
    /// Type of transaction (e.g., "DEAL", "WITH")
    pub transaction_type: String,
    /// Profit and loss in EUR
    pub pnl_eur: f64,
    /// Unique reference for the transaction
    pub reference: String,
    /// Whether this transaction is a fee
    pub is_fee: bool,
    /// Original JSON string of the transaction
    pub raw_json: String,
}

impl_json_display!(StoreTransaction);

impl From<AccountTransaction> for StoreTransaction {
    fn from(raw: AccountTransaction) -> Self {
        fn parse_period(period: &str) -> Option<NaiveDate> {
            // For format "DD-MON-YY"
            if let Some((day_str, rest)) = period.split_once('-')
                && let Some((mon_str, year_str)) = rest.split_once('-')
            {
                // Try to parse the day
                if let Ok(day) = day_str.parse::<u32>() {
                    let month = chrono::Month::from_str(mon_str).ok()?;
                    let year = 2000 + year_str.parse::<i32>().ok()?;

                    // Return the exact date
                    return NaiveDate::from_ymd_opt(year, month.number_from_month(), day);
                }
            }

            // For format "MON-YY"
            if let Some((mon_str, year_str)) = period.split_once('-') {
                let month = chrono::Month::from_str(mon_str).ok()?;
                let year = 2000 + year_str.parse::<i32>().ok()?;

                // Get the first day of the month
                let first_of_month = NaiveDate::from_ymd_opt(year, month.number_from_month(), 1)?;

                // Get the first day of the previous month
                let prev_month = if month.number_from_month() == 1 {
                    // If January, go to December of previous year
                    NaiveDate::from_ymd_opt(year - 1, 12, 1)?
                } else {
                    // Otherwise, just go to previous month
                    NaiveDate::from_ymd_opt(year, month.number_from_month() - 1, 1)?
                };

                // Find the last day of the previous month
                let last_day_of_prev_month = if prev_month.month() == 12 {
                    // December has 31 days
                    NaiveDate::from_ymd_opt(prev_month.year(), 12, 31)?
                } else {
                    // For other months, the last day is one day before the first of current month
                    first_of_month - Duration::days(1)
                };

                // Calculate how many days to go back to find the last Wednesday
                let days_back = (last_day_of_prev_month.weekday().num_days_from_monday() + 7
                    - Weekday::Wed.num_days_from_monday())
                    % 7;

                // Get the last Wednesday
                return Some(last_day_of_prev_month - Duration::days(days_back as i64));
            }

            None
        }

        let instrument_info: ParsedOptionInfo = parse_instrument_name(&raw.instrument_name);
        let underlying = Some(instrument_info.asset_name);
        let strike = match instrument_info {
            ParsedOptionInfo {
                strike: Some(s), ..
            } => Some(s.parse::<f64>().ok()).flatten(),
            _ => None,
        };
        let option_type = instrument_info.option_type;
        let deal_date = NaiveDateTime::parse_from_str(&raw.date_utc, "%Y-%m-%dT%H:%M:%S")
            .map(|naive| naive.and_utc())
            .unwrap_or_else(|_| Utc::now());
        let pnl_eur = raw
            .profit_and_loss
            .trim_start_matches('E')
            .parse::<f64>()
            .unwrap_or(0.0);

        let expiry = parse_period(&raw.period);

        let is_fee = raw.transaction_type == "WITH" && pnl_eur.abs() < 1.0;

        StoreTransaction {
            deal_date,
            underlying,
            strike,
            option_type,
            expiry,
            transaction_type: raw.transaction_type.clone(),
            pnl_eur,
            reference: raw.reference.clone(),
            is_fee,
            raw_json: raw.to_string(),
        }
    }
}

impl From<&AccountTransaction> for StoreTransaction {
    fn from(raw: &AccountTransaction) -> Self {
        StoreTransaction::from(raw.clone())
    }
}

/// Collection of processed transactions from IG Markets
///
/// This struct is a wrapper around a vector of `StoreTransaction` objects
/// and provides convenient methods for accessing and converting transaction data.
pub struct TransactionList(pub Vec<StoreTransaction>);

impl AsRef<[StoreTransaction]> for TransactionList {
    fn as_ref(&self) -> &[StoreTransaction] {
        &self.0[..]
    }
}

impl From<&Vec<AccountTransaction>> for TransactionList {
    fn from(raw: &Vec<AccountTransaction>) -> Self {
        TransactionList(
            raw.iter() // Usa iter() en lugar de into_iter() para referencias
                .map(StoreTransaction::from) // Esto asume que hay un impl From<&AccountTransaction> for StoreTransaction
                .collect(),
        )
    }
}
