use crate::application::services::MarketService;
use crate::application::services::types::DBEntry;
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
use std::collections::{HashMap, HashSet};
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
    /// Navigates through all levels of the market hierarchy and collects all MarketData
    ///
    /// This method performs a comprehensive traversal of the IG Markets hierarchy,
    /// starting from the root navigation and going through multiple levels to collect
    /// all available market instruments.
    ///
    /// # Arguments
    /// * `session` - The authenticated IG session
    /// * `max_levels` - Maximum depth to traverse (default: 5 levels)
    ///
    /// # Returns
    /// * `Result<Vec<MarketData>, AppError>` - Vector containing all found market instruments
    ///
    async fn get_all_markets(
        &self,
        session: &IgSession,
    ) -> Result<Vec<crate::application::models::market::MarketData>, AppError> {
        let max_depth = 6;
        info!("Starting comprehensive market hierarchy traversal (max {} levels)", max_depth);
        
        // Get the root navigation
        let root_response = self.get_market_navigation(session).await?;
        info!(
            "Root navigation: {} nodes, {} markets at top level",
            root_response.nodes.len(),
            root_response.markets.len()
        );
        
        // Start with markets from the root level
        let mut all_markets = root_response.markets.clone();
        
        // Use iterative approach to navigate through all levels
        let mut nodes_to_process = root_response.nodes.clone();
        let mut processed_levels = 0;
        
        while !nodes_to_process.is_empty() && processed_levels < max_depth {
            let mut next_level_nodes = Vec::new();
            let mut level_market_count = 0;
            
            info!("Processing level {} with {} nodes", processed_levels, nodes_to_process.len());
            
            for node in &nodes_to_process {
                match self.get_market_navigation_node(session, &node.id).await {
                    Ok(node_response) => {
                        let node_markets = node_response.markets.len();
                        let node_children = node_response.nodes.len();
                        
                        if node_markets > 0 || node_children > 0 {
                            debug!(
                                "Node '{}' (level {}): {} markets, {} child nodes",
                                node.name, processed_levels, node_markets, node_children
                            );
                        }
                        
                        // Add markets from this node
                        all_markets.extend(node_response.markets);
                        level_market_count += node_markets;
                        
                        // Add child nodes for next level processing
                        next_level_nodes.extend(node_response.nodes);
                    }
                    Err(e) => {
                        error!(
                            "Failed to get markets for node '{}' at level {}: {:?}",
                            node.name, processed_levels, e
                        );
                    }
                }
            }
            
            info!(
                "Level {} completed: {} markets found, {} nodes for next level",
                processed_levels, level_market_count, next_level_nodes.len()
            );
            
            nodes_to_process = next_level_nodes;
            processed_levels += 1;
        }
        
        info!(
            "Market hierarchy traversal completed: {} total markets found across {} levels",
            all_markets.len(),
            processed_levels
        );
        
        Ok(all_markets)
    }

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
        info!("Getting all markets from hierarchy for DB entries");
        
        // Use the get_all_markets method to collect all markets from the hierarchy
        let all_markets = self.get_all_markets(session).await?;
        
        info!("Collected {} markets from hierarchy", all_markets.len());

        // Convert all collected markets to DBEntry
        let mut vec_db_entries: Vec<DBEntry> = all_markets
            .iter()
            .map(|market| DBEntry::from(market))
            .filter(|entry| !entry.epic.is_empty()) // Filter entries that HAVE epics
            .collect();

        info!("Created {} DB entries from markets", vec_db_entries.len());

        // Update the expiry date in each DBEntry
        // All entries with the same symbol share the same expiry date
        // Get the proper expiry date from self.get_market_details
        // MarketDetails.instrument.expiry_details.last_dealing_date

        // Create a hash map with the symbol as key and the expiry date as value
        let mut symbol_expiry_map: HashMap<String, String> = HashMap::new();

        // Collect unique symbols to avoid duplicate API calls
        let unique_symbols: HashSet<String> = vec_db_entries
            .iter()
            .map(|entry| entry.symbol.clone())
            .filter(|symbol| !symbol.is_empty())
            .collect();

        info!(
            "Found {} unique symbols to fetch expiry dates for",
            unique_symbols.len()
        );

        // Fetch expiry dates for each unique symbol
        for symbol in unique_symbols {
            // Find the first epic for this symbol to get market details
            if let Some(entry) = vec_db_entries
                .iter()
                .find(|e| e.symbol == symbol && !e.epic.is_empty())
            {
                match self.get_market_details(session, &entry.epic).await {
                    Ok(market_details) => {
                        // Extract expiry date from market details
                        let expiry_date = market_details
                            .instrument
                            .expiry_details
                            .as_ref()
                            .map(|details| details.last_dealing_date.clone())
                            .unwrap_or_else(|| {
                                // Fallback to the expiry field from the instrument if expiry_details is not available
                                market_details.instrument.expiry.clone()
                            });

                        symbol_expiry_map.insert(symbol.clone(), expiry_date);
                        info!(
                            "Fetched expiry date for symbol {}: {}",
                            symbol,
                            symbol_expiry_map.get(&symbol).unwrap()
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to get market details for epic {} (symbol {}): {:?}",
                            entry.epic, symbol, e
                        );
                        // Use the existing expiry from the entry as fallback
                        symbol_expiry_map.insert(symbol.clone(), entry.expiry.clone());
                    }
                }
            }
        }

        // Update vec_db_entries.expiry with the value from the hash map
        for entry in &mut vec_db_entries {
            if let Some(expiry_date) = symbol_expiry_map.get(&entry.symbol) {
                entry.expiry = expiry_date.clone();
            }
        }

        info!("Updated expiry dates for {} entries", vec_db_entries.len());
        Ok(vec_db_entries)
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
