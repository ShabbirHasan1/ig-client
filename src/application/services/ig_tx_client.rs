use crate::application::models::transaction::{RawTransaction, Transaction};
use crate::config::Config;
use crate::error::AppError;
use crate::session::interface::IgSession;
use crate::utils::parsing::{InstrumentInfo, parse_instrument_name};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use reqwest::{Client, StatusCode};
use std::str::FromStr;
use tracing::debug;

#[async_trait]
pub trait IgTxFetcher {
    async fn fetch_range(
        &self,
        sess: &IgSession,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Transaction>, AppError>;
}

pub struct IgTxClient<'a> {
    cfg: &'a Config,
    http: Client,
}

impl<'a> IgTxClient<'a> {
    pub fn new(cfg: &'a Config) -> Self {
        Self {
            cfg,
            http: Client::builder()
                .user_agent("ig-rs/0.1")
                .build()
                .expect("reqwest"),
        }
    }

    #[allow(dead_code)]
    fn rest_url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.cfg.rest_api.base_url.trim_end_matches('/'),
            path
        )
    }

    fn convert(&self, raw: RawTransaction) -> Result<Transaction, AppError> {
        let instrument_info: InstrumentInfo = parse_instrument_name(&raw.instrument_name)?;
        let underlying = instrument_info.underlying;
        let strike = instrument_info.strike;
        let option_type = instrument_info.option_type;

        let deal_date = NaiveDateTime::parse_from_str(&raw.date_utc, "%Y-%m-%dT%H:%M:%S")
            .map(|naive| naive.and_utc())
            .unwrap_or_else(|_| Utc::now());

        let pnl_eur = raw
            .pnl_raw
            .trim_start_matches('E')
            .parse::<f64>()
            .unwrap_or(0.0);

        let expiry = raw.period.split_once('-').and_then(|(mon, yy)| {
            chrono::Month::from_str(mon).ok().and_then(|m| {
                NaiveDate::from_ymd_opt(2000 + yy.parse::<i32>().ok()?, m.number_from_month(), 1)
            })
        });

        let is_fee = raw.transaction_type == "WITH" && pnl_eur.abs() < 1.0;

        Ok(Transaction {
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
        })
    }
}

#[async_trait]
impl IgTxFetcher for IgTxClient<'_> {
    async fn fetch_range(
        &self,
        sess: &IgSession,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Transaction>, AppError> {
        let mut page = 1;
        let mut out = Vec::new();

        loop {
            let url = format!(
                "{}/history/transactions?from={}&to={}&pageNumber={}&pageSize=200",
                self.cfg.rest_api.base_url,
                from.format("%Y-%m-%dT%H:%M:%S"),
                to.format("%Y-%m-%dT%H:%M:%S"),
                page
            );
            debug!("🔗 Fetching IG txs from URL: {}", url);

            let resp = self
                .http
                .get(&url)
                .header("X-IG-API-KEY", &self.cfg.credentials.api_key)
                .header("CST", &sess.cst)
                .header("X-SECURITY-TOKEN", &sess.token)
                .header("Version", "2")
                .header("Accept", "application/json; charset=UTF-8")
                .send()
                .await?;

            if resp.status() != StatusCode::OK {
                return Err(AppError::Unexpected(resp.status()));
            }

            let json: serde_json::Value = resp.json().await?;
            let raws: Vec<RawTransaction> =
                serde_json::from_value(json["transactions"].clone()).unwrap_or_default();

            if raws.is_empty() {
                break;
            }

            out.extend(raws.into_iter().map(|r| self.convert(r).unwrap()));

            let meta = &json["metadata"]["pageData"];
            let total_pages = meta["totalPages"].as_u64().unwrap_or(1);
            if page >= total_pages {
                break;
            }
            page += 1;
        }

        Ok(out)
    }
}
