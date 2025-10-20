/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use crate::application::interfaces::market::MarketService;
use crate::error::AppError;
use crate::model::http::HttpClient;
use crate::model::requests::RecentPricesRequest;
use crate::model::responses::{DBEntryResponse, MultipleMarketDetailsResponse};
use crate::presentation::market::{
    HistoricalPricesResponse, MarketData, MarketDetails, MarketNavigationResponse,
    MarketSearchResult,
};
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, info};

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
        let path = format!("markets?searchTerm={}", search_term);
        info!("Searching markets with term: {}", search_term);
        let result: MarketSearchResult = self.http_client.get(&path, Some(1)).await?;
        debug!("{} markets found", result.markets.len());
        Ok(result)
    }

    async fn get_market_details(&self, epic: &str) -> Result<MarketDetails, AppError> {
        let path = format!("markets/{epic}");
        info!("Getting market details: {}", epic);
        let market_value: Value = self.http_client.get(&path, Some(3)).await?;
        let market_details: MarketDetails = serde_json::from_value(market_value)?;
        debug!("Market details obtained for: {}", epic);
        Ok(market_details)
    }

    async fn get_multiple_market_details(
        &self,
        epics: &[String],
    ) -> Result<MultipleMarketDetailsResponse, AppError> {
        if epics.is_empty() {
            return Ok(Vec::new());
        } else if epics.len() > 50 {
            return Err(AppError::InvalidInput(
                "The maximum number of EPICs is 50".to_string(),
            ));
        }

        let epics_str = epics.join(",");
        let path = format!("markets?epics={}", epics_str);
        debug!(
            "Getting market details for {} EPICs in a batch",
            epics.len()
        );
        
        let response: MultipleMarketDetailsResponse = self.http_client.get(&path, Some(2)).await?;

        Ok(response)
    }

    async fn get_historical_prices(
        &self,
        epic: &str,
        resolution: &str,
        from: &str,
        to: &str,
    ) -> Result<HistoricalPricesResponse, AppError> {
        let path = format!(
            "prices/{}?resolution={}&from={}&to={}",
            epic, resolution, from, to
        );
        info!("Getting historical prices for: {}", epic);
        let result: HistoricalPricesResponse = self.http_client.get(&path, Some(3)).await?;
        debug!("Historical prices obtained for: {}", epic);
        Ok(result)
    }

    async fn get_historical_prices_by_date_range(
        &self,
        epic: &str,
        resolution: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<HistoricalPricesResponse, AppError> {
        let path = format!("prices/{}/{}/{}/{}", epic, resolution, start_date, end_date);
        info!(
            "Getting historical prices for epic: {}, resolution: {}, from: {} to: {}",
            epic, resolution, start_date, end_date
        );
        let result: HistoricalPricesResponse = self.http_client.get(&path, Some(2)).await?;
        debug!(
            "Historical prices obtained for epic: {}, {} data points",
            epic,
            result.prices.len()
        );
        Ok(result)
    }

    async fn get_recent_prices(
        &self,
        _params: &RecentPricesRequest<'_>,
    ) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_historical_prices_by_count_v1(
        &self,
        _epic: &str,
        _resolution: &str,
        _num_points: i32,
    ) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_historical_prices_by_count_v2(
        &self,
        _epic: &str,
        _resolution: &str,
        _num_points: i32,
    ) -> Result<HistoricalPricesResponse, AppError> {
        todo!()
    }

    async fn get_market_navigation(&self) -> Result<MarketNavigationResponse, AppError> {
        todo!()
    }

    async fn get_market_navigation_node(
        &self,
        _node_id: &str,
    ) -> Result<MarketNavigationResponse, AppError> {
        todo!()
    }

    async fn get_all_markets(&self) -> Result<Vec<MarketData>, AppError> {
        todo!()
    }

    async fn get_vec_db_entries(&self) -> Result<Vec<DBEntryResponse>, AppError> {
        todo!()
    }
}
