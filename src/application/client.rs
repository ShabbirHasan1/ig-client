/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};
use tracing_subscriber::util::SubscriberInitExt;
use crate::application::interfaces::market::MarketService;
use crate::error::AppError;
use crate::model::http::HttpClient;
use crate::model::requests::RecentPricesRequest;
use crate::model::responses::DBEntryResponse;
use crate::presentation::market::{HistoricalPricesResponse, MarketData, MarketDetails, MarketNavigationResponse, MarketSearchResult};

pub struct Client {
    http_client: Arc<HttpClient>,
}

impl Client {
    pub fn new() -> Self {
        let http_client = Arc::new(HttpClient::default());
        let _ = http_client.get_session();
        Self { http_client }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketService for Client {
    async fn search_markets(&self, search_term: &str) -> Result<MarketSearchResult, AppError> {
        todo!()
    }

    async fn get_market_details(&self, epic: &str) -> Result<MarketDetails, AppError> {
        let path = format!("markets/{epic}");
        info!("Getting market details: {}", epic);
        let market_value: Value = self.http_client.get(&path,Some(3)).await?;
        let market_details: MarketDetails = serde_json::from_value(market_value)?;
        debug!("Market details obtained for: {}", epic);
        Ok(market_details)
    }

    async fn get_multiple_market_details(&self, epics: &[String]) -> Result<Vec<MarketDetails>, AppError> {
        todo!()
    }

    async fn get_historical_prices(&self, epic: &str, resolution: &str, from: &str, to: &str) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_historical_prices_by_date_range(&self, epic: &str, resolution: &str, start_date: &str, end_date: &str) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_recent_prices(&self, params: &RecentPricesRequest<'_>) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_historical_prices_by_count_v1(&self, epic: &str, resolution: &str, num_points: i32) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_historical_prices_by_count_v2(&self, epic: &str, resolution: &str, num_points: i32) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_market_navigation(&self) -> Result<MarketNavigationResponse, AppError> {
        todo!()
    }

    async fn get_market_navigation_node(&self, node_id: &str) -> Result<MarketNavigationResponse, AppError> {
        todo!()
    }

    async fn get_all_markets(&self) -> Result<Vec<MarketData>, AppError> {
        todo!()
    }

    async fn get_vec_db_entries(&self) -> Result<Vec<DBEntryResponse>, AppError> {
        todo!()
    }
}