use crate::application::models::market::MarketNode;
use crate::application::services::MarketService;
use crate::application::services::types::DBEntry;
use crate::presentation::build_market_hierarchy;
use crate::{
    application::models::market::{
        HistoricalPricesResponse, MarketDetails, MarketNavigationResponse, MarketSearchResult,
    },
    config::Config,
    error::AppError,
    session::interface::IgSession,
    transport::http_client::IgHttpClient,
};
use async_trait::async_trait;
use reqwest::Method;
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Implementation of the market service
pub struct MarketServiceImpl<T: IgHttpClient> {
    config: Arc<Config>,
    client: Arc<T>,
}

impl<T: IgHttpClient> MarketServiceImpl<T> {
    /// Creates a new instance of the market service
    pub fn new(config: Arc<Config>, client: Arc<T>) -> Self {
        Self { config, client }
    }

    /// Gets the current configuration
    ///
    /// # Returns
    /// * Reference to the current configuration
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Sets a new configuration
    ///
    /// # Arguments
    /// * `config` - The new configuration to use
    pub fn set_config(&mut self, config: Arc<Config>) {
        self.config = config;
    }
}

#[async_trait]
impl<T: IgHttpClient + 'static> MarketService for MarketServiceImpl<T> {
    async fn search_markets(
        &self,
        session: &IgSession,
        search_term: &str,
    ) -> Result<MarketSearchResult, AppError> {
        let path = format!("markets?searchTerm={search_term}");
        info!("Searching markets with term: {}", search_term);

        let result = self
            .client
            .request::<(), MarketSearchResult>(Method::GET, &path, session, None, "1")
            .await?;

        debug!("{} markets found", result.markets.len());
        Ok(result)
    }

    async fn get_market_details(
        &self,
        session: &IgSession,
        epic: &str,
    ) -> Result<MarketDetails, AppError> {
        let path = format!("markets/{epic}");
        info!("Getting market details: {}", epic);

        let result = self
            .client
            .request::<(), MarketDetails>(Method::GET, &path, session, None, "3")
            .await?;

        debug!("Market details obtained for: {}", epic);
        Ok(result)
    }

    async fn get_multiple_market_details(
        &self,
        session: &IgSession,
        epics: &[String],
    ) -> Result<Vec<MarketDetails>, AppError> {
        if epics.is_empty() {
            return Ok(Vec::new());
        } else if epics.len() > 50 {
            return Err(AppError::InvalidInput(
                "The maximum number of EPICs is 50".to_string(),
            ));
        }

        // Join the EPICs with commas to create a single request
        let epics_str = epics.join(",");
        let path = format!("markets?epics={epics_str}");

        debug!(
            "Getting market details for {} EPICs in a batch: {}",
            epics.len(),
            epics_str
        );

        // The API returns an object with un array de MarketDetails en la propiedad marketDetails
        #[derive(serde::Deserialize)]
        struct MarketDetailsResponse {
            #[serde(rename = "marketDetails")]
            market_details: Vec<MarketDetails>,
        }

        let response = self
            .client
            .request::<(), MarketDetailsResponse>(Method::GET, &path, session, None, "2")
            .await?;

        debug!(
            "Market details obtained for {} EPICs",
            response.market_details.len()
        );
        Ok(response.market_details)
    }

    async fn get_historical_prices(
        &self,
        session: &IgSession,
        epic: &str,
        resolution: &str,
        from: &str,
        to: &str,
    ) -> Result<HistoricalPricesResponse, AppError> {
        let path = format!("prices/{epic}?resolution={resolution}&from={from}&to={to}");
        info!("Getting historical prices for: {}", epic);

        let result = self
            .client
            .request::<(), HistoricalPricesResponse>(Method::GET, &path, session, None, "3")
            .await?;

        debug!("Historical prices obtained for: {}", epic);
        Ok(result)
    }

    async fn get_market_navigation(
        &self,
        session: &IgSession,
    ) -> Result<MarketNavigationResponse, AppError> {
        let path = "marketnavigation";
        info!("Getting top-level market navigation nodes");

        let result = self
            .client
            .request::<(), MarketNavigationResponse>(Method::GET, path, session, None, "1")
            .await?;

        debug!("{} navigation nodes found", result.nodes.len());
        debug!("{} markets found at root level", result.markets.len());
        Ok(result)
    }

    async fn get_market_navigation_node(
        &self,
        session: &IgSession,
        node_id: &str,
    ) -> Result<MarketNavigationResponse, AppError> {
        let path = format!("marketnavigation/{node_id}");
        info!("Getting market navigation node: {}", node_id);

        let result = self
            .client
            .request::<(), MarketNavigationResponse>(Method::GET, &path, session, None, "1")
            .await?;

        debug!("{} child nodes found", result.nodes.len());
        debug!("{} markets found in node {}", result.markets.len(), node_id);
        Ok(result)
    }

    async fn get_vec_db_entries(&self, session: &IgSession) -> Result<Vec<DBEntry>, AppError> {
        match self.get_market_navigation(&session).await {
            Ok(response) => {
                info!(
                    "Test successful: {} nodes, {} markets at top level",
                    response.nodes.len(),
                    response.markets.len()
                );

                // If the test is successful, build the complete hierarchy
                info!("Building market hierarchy...");
                let hierarchy = match build_market_hierarchy(&self, &session, None, 0).await {
                    Ok(h) => {
                        info!(
                            "Successfully built hierarchy with {} top-level nodes",
                            h.len()
                        );
                        h
                    }
                    Err(e) => {
                        error!("Error building complete hierarchy: {:?}", e);
                        info!("Attempting to build a partial hierarchy with rate limiting...");
                        // Try again with a smaller scope
                        let limited_nodes = response
                            .nodes
                            .iter()
                            .map(|n| MarketNode {
                                id: n.id.clone(),
                                name: n.name.clone(),
                                children: Vec::new(),
                                markets: Vec::new(),
                            })
                            .collect::<Vec<_>>();
                        info!(
                            "Created partial hierarchy with {} top-level nodes",
                            limited_nodes.len()
                        );
                        limited_nodes
                    }
                };

                let vec_db_entries: Vec<DBEntry> = hierarchy
                    .iter()
                    .map(|node| DBEntry::from(node))
                    .filter(|entry| entry.epic.is_empty())
                    .collect();
                
                // TODO: update the expiry date in each DBEntry
                // all entries with the same symbol share the same expiry date
                // we should get the proper expiry date from self.get_market_details
                // MarketDetails.Instrument.expiry_details.last_dealing_date
                // create a hash map with the symbol as key and the expiry date as value
                Ok(vec_db_entries)
            }
            Err(e) => {
                error!("Failed to get market navigation: {:?}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::transport::http_client::IgHttpClientImpl;
    use crate::utils::rate_limiter::RateLimitType;
    use std::sync::Arc;

    #[test]
    fn test_get_and_set_config() {
        let config = Arc::new(Config::with_rate_limit_type(
            RateLimitType::NonTradingAccount,
            0.7,
        ));
        let client = Arc::new(IgHttpClientImpl::new(config.clone()));
        let mut service = MarketServiceImpl::new(config.clone(), client.clone());
        assert!(std::ptr::eq(service.get_config(), &*config));
        let new_cfg = Arc::new(Config::default());
        service.set_config(new_cfg.clone());
        assert!(std::ptr::eq(service.get_config(), &*new_cfg));
    }
}
