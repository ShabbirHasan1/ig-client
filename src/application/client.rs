/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/
use crate::application::interfaces::account::AccountService;
use crate::application::interfaces::market::MarketService;
use crate::application::interfaces::order::OrderService;
use crate::error::AppError;
use crate::model::http::HttpClient;
use crate::model::requests::RecentPricesRequest;
use crate::model::responses::{
    DBEntryResponse, HistoricalPricesResponse, MarketNavigationResponse, MarketSearchResponse,
    MultipleMarketDetailsResponse,
};
use crate::prelude::{
    AccountActivityResponse, AccountsResponse, PositionsResponse, TransactionHistoryResponse,
    WorkingOrdersResponse,
};
use crate::presentation::market::{MarketData, MarketDetails};
use crate::presentation::order::{
    ClosePositionRequest, ClosePositionResponse, CreateOrderRequest, CreateOrderResponse,
    OrderConfirmation, UpdatePositionRequest, UpdatePositionResponse,
};
use crate::presentation::working_order::{CreateWorkingOrderRequest, CreateWorkingOrderResponse};
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
    async fn search_markets(&self, search_term: &str) -> Result<MarketSearchResponse, AppError> {
        let path = format!("markets?searchTerm={}", search_term);
        info!("Searching markets with term: {}", search_term);
        let result: MarketSearchResponse = self.http_client.get(&path, Some(1)).await?;
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
            return Ok(MultipleMarketDetailsResponse::default());
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
        params: &RecentPricesRequest<'_>,
    ) -> Result<HistoricalPricesResponse, AppError> {
        let mut query_params = Vec::new();

        if let Some(res) = params.resolution {
            query_params.push(format!("resolution={}", res));
        }
        if let Some(f) = params.from {
            query_params.push(format!("from={}", f));
        }
        if let Some(t) = params.to {
            query_params.push(format!("to={}", t));
        }
        if let Some(max) = params.max_points {
            query_params.push(format!("max={}", max));
        }
        if let Some(size) = params.page_size {
            query_params.push(format!("pageSize={}", size));
        }
        if let Some(num) = params.page_number {
            query_params.push(format!("pageNumber={}", num));
        }

        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        let path = format!("prices/{}{}", params.epic, query_string);
        info!("Getting recent prices for epic: {}", params.epic);
        let result: HistoricalPricesResponse = self.http_client.get(&path, Some(3)).await?;
        debug!(
            "Recent prices obtained for epic: {}, {} data points",
            params.epic,
            result.prices.len()
        );
        Ok(result)
    }

    async fn get_historical_prices_by_count_v1(
        &self,
        epic: &str,
        resolution: &str,
        num_points: i32,
    ) -> Result<HistoricalPricesResponse, AppError> {
        let path = format!("prices/{}/{}/{}", epic, resolution, num_points);
        info!(
            "Getting historical prices (v1) for epic: {}, resolution: {}, points: {}",
            epic, resolution, num_points
        );
        let result: HistoricalPricesResponse = self.http_client.get(&path, Some(1)).await?;
        debug!(
            "Historical prices (v1) obtained for epic: {}, {} data points",
            epic,
            result.prices.len()
        );
        Ok(result)
    }

    async fn get_historical_prices_by_count_v2(
        &self,
        epic: &str,
        resolution: &str,
        num_points: i32,
    ) -> Result<HistoricalPricesResponse, AppError> {
        let path = format!("prices/{}/{}/{}", epic, resolution, num_points);
        info!(
            "Getting historical prices (v2) for epic: {}, resolution: {}, points: {}",
            epic, resolution, num_points
        );
        let result: HistoricalPricesResponse = self.http_client.get(&path, Some(2)).await?;
        debug!(
            "Historical prices (v2) obtained for epic: {}, {} data points",
            epic,
            result.prices.len()
        );
        Ok(result)
    }

    async fn get_market_navigation(&self) -> Result<MarketNavigationResponse, AppError> {
        let path = "marketnavigation";
        info!("Getting top-level market navigation nodes");
        let result: MarketNavigationResponse = self.http_client.get(path, Some(1)).await?;
        debug!("{} navigation nodes found", result.nodes.len());
        debug!("{} markets found at root level", result.markets.len());
        Ok(result)
    }

    async fn get_market_navigation_node(
        &self,
        node_id: &str,
    ) -> Result<MarketNavigationResponse, AppError> {
        let path = format!("marketnavigation/{}", node_id);
        info!("Getting market navigation node: {}", node_id);
        let result: MarketNavigationResponse = self.http_client.get(&path, Some(1)).await?;
        debug!("{} child nodes found", result.nodes.len());
        debug!("{} markets found in node {}", result.markets.len(), node_id);
        Ok(result)
    }

    async fn get_all_markets(&self) -> Result<Vec<MarketData>, AppError> {
        let max_depth = 6;
        info!(
            "Starting comprehensive market hierarchy traversal (max {} levels)",
            max_depth
        );

        let root_response = self.get_market_navigation().await?;
        info!(
            "Root navigation: {} nodes, {} markets at top level",
            root_response.nodes.len(),
            root_response.markets.len()
        );

        let mut all_markets = root_response.markets.clone();
        let mut nodes_to_process = root_response.nodes.clone();
        let mut processed_levels = 0;

        while !nodes_to_process.is_empty() && processed_levels < max_depth {
            let mut next_level_nodes = Vec::new();
            let mut level_market_count = 0;

            info!(
                "Processing level {} with {} nodes",
                processed_levels,
                nodes_to_process.len()
            );

            for node in &nodes_to_process {
                match self.get_market_navigation_node(&node.id).await {
                    Ok(node_response) => {
                        let node_markets = node_response.markets.len();
                        let node_children = node_response.nodes.len();

                        if node_markets > 0 || node_children > 0 {
                            debug!(
                                "Node '{}' (level {}): {} markets, {} child nodes",
                                node.name, processed_levels, node_markets, node_children
                            );
                        }

                        all_markets.extend(node_response.markets);
                        level_market_count += node_markets;
                        next_level_nodes.extend(node_response.nodes);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to get markets for node '{}' at level {}: {:?}",
                            node.name,
                            processed_levels,
                            e
                        );
                    }
                }
            }

            info!(
                "Level {} completed: {} markets found, {} nodes for next level",
                processed_levels,
                level_market_count,
                next_level_nodes.len()
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

    async fn get_vec_db_entries(&self) -> Result<Vec<DBEntryResponse>, AppError> {
        info!("Getting all markets from hierarchy for DB entries");

        let all_markets = self.get_all_markets().await?;
        info!("Collected {} markets from hierarchy", all_markets.len());

        let mut vec_db_entries: Vec<DBEntryResponse> = all_markets
            .iter()
            .map(DBEntryResponse::from)
            .filter(|entry| !entry.epic.is_empty())
            .collect();

        info!("Created {} DB entries from markets", vec_db_entries.len());

        // Collect unique symbols
        let unique_symbols: std::collections::HashSet<String> = vec_db_entries
            .iter()
            .map(|entry| entry.symbol.clone())
            .filter(|symbol| !symbol.is_empty())
            .collect();

        info!(
            "Found {} unique symbols to fetch expiry dates for",
            unique_symbols.len()
        );

        let mut symbol_expiry_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for symbol in unique_symbols {
            if let Some(entry) = vec_db_entries
                .iter()
                .find(|e| e.symbol == symbol && !e.epic.is_empty())
            {
                match self.get_market_details(&entry.epic).await {
                    Ok(market_details) => {
                        let expiry_date = market_details
                            .instrument
                            .expiry_details
                            .as_ref()
                            .map(|details| details.last_dealing_date.clone())
                            .unwrap_or_else(|| market_details.instrument.expiry.clone());

                        symbol_expiry_map.insert(symbol.clone(), expiry_date);
                        info!(
                            "Fetched expiry date for symbol {}: {}",
                            symbol,
                            symbol_expiry_map.get(&symbol).unwrap()
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to get market details for epic {} (symbol {}): {:?}",
                            entry.epic,
                            symbol,
                            e
                        );
                        symbol_expiry_map.insert(symbol.clone(), entry.expiry.clone());
                    }
                }
            }
        }

        for entry in &mut vec_db_entries {
            if let Some(expiry_date) = symbol_expiry_map.get(&entry.symbol) {
                entry.expiry = expiry_date.clone();
            }
        }

        info!("Updated expiry dates for {} entries", vec_db_entries.len());
        Ok(vec_db_entries)
    }
}

#[async_trait]
impl AccountService for Client {
    async fn get_accounts(&self) -> Result<AccountsResponse, AppError> {
        info!("Getting account information");
        let result: AccountsResponse = self.http_client.get("accounts", Some(1)).await?;
        debug!(
            "Account information obtained: {} accounts",
            result.accounts.len()
        );
        Ok(result)
    }

    async fn get_positions(&self) -> Result<PositionsResponse, AppError> {
        debug!("Getting open positions");
        let result: PositionsResponse = self.http_client.get("positions", Some(2)).await?;
        debug!("Positions obtained: {} positions", result.positions.len());
        Ok(result)
    }

    async fn get_positions_w_filter(&self, filter: &str) -> Result<PositionsResponse, AppError> {
        debug!("Getting open positions with filter: {}", filter);
        let mut positions = self.get_positions().await?;

        positions
            .positions
            .retain(|position| position.market.epic.contains(filter));

        debug!(
            "Positions obtained after filtering: {} positions",
            positions.positions.len()
        );
        Ok(positions)
    }

    async fn get_working_orders(&self) -> Result<WorkingOrdersResponse, AppError> {
        info!("Getting working orders");
        let result: WorkingOrdersResponse = self.http_client.get("workingorders", Some(2)).await?;
        debug!(
            "Working orders obtained: {} orders",
            result.working_orders.len()
        );
        Ok(result)
    }

    async fn get_activity(
        &self,
        from: &str,
        to: &str,
    ) -> Result<AccountActivityResponse, AppError> {
        let path = format!("history/activity?from={}&to={}&pageSize=500", from, to);
        info!("Getting account activity");
        let result: AccountActivityResponse = self.http_client.get(&path, Some(3)).await?;
        debug!(
            "Account activity obtained: {} activities",
            result.activities.len()
        );
        Ok(result)
    }

    async fn get_activity_with_details(
        &self,
        from: &str,
        to: &str,
    ) -> Result<AccountActivityResponse, AppError> {
        let path = format!(
            "history/activity?from={}&to={}&detailed=true&pageSize=500",
            from, to
        );
        info!("Getting detailed account activity");
        let result: AccountActivityResponse = self.http_client.get(&path, Some(3)).await?;
        debug!(
            "Detailed account activity obtained: {} activities",
            result.activities.len()
        );
        Ok(result)
    }

    async fn get_transactions(
        &self,
        from: &str,
        to: &str,
    ) -> Result<TransactionHistoryResponse, AppError> {
        const PAGE_SIZE: u32 = 200;
        let mut all_transactions = Vec::new();
        let mut current_page = 1;
        #[allow(unused_assignments)]
        let mut last_metadata = None;

        loop {
            let path = format!(
                "history/transactions?from={}&to={}&pageSize={}&pageNumber={}",
                from, to, PAGE_SIZE, current_page
            );
            info!("Getting transaction history page {}", current_page);

            let result: TransactionHistoryResponse = self.http_client.get(&path, Some(2)).await?;

            let total_pages = result.metadata.page_data.total_pages as u32;
            last_metadata = Some(result.metadata);
            all_transactions.extend(result.transactions);

            if current_page >= total_pages {
                break;
            }
            current_page += 1;
        }

        debug!(
            "Total transaction history obtained: {} transactions",
            all_transactions.len()
        );

        Ok(TransactionHistoryResponse {
            transactions: all_transactions,
            metadata: last_metadata
                .ok_or_else(|| AppError::InvalidInput("Could not retrieve metadata".to_string()))?,
        })
    }
}

#[async_trait]
impl OrderService for Client {
    async fn create_order(
        &self,
        order: &CreateOrderRequest,
    ) -> Result<CreateOrderResponse, AppError> {
        info!("Creating order for: {}", order.epic);
        let result: CreateOrderResponse = self
            .http_client
            .post("positions/otc", order, Some(2))
            .await?;
        debug!("Order created with reference: {}", result.deal_reference);
        Ok(result)
    }

    async fn get_order_confirmation(
        &self,
        deal_reference: &str,
    ) -> Result<OrderConfirmation, AppError> {
        let path = format!("confirms/{}", deal_reference);
        info!("Getting confirmation for order: {}", deal_reference);
        let result: OrderConfirmation = self.http_client.get(&path, Some(1)).await?;
        debug!("Confirmation obtained for order: {}", deal_reference);
        Ok(result)
    }

    async fn update_position(
        &self,
        deal_id: &str,
        update: &UpdatePositionRequest,
    ) -> Result<UpdatePositionResponse, AppError> {
        let path = format!("positions/otc/{}", deal_id);
        info!("Updating position: {}", deal_id);
        let result: UpdatePositionResponse = self.http_client.put(&path, update, Some(2)).await?;
        debug!(
            "Position updated: {} with deal reference: {}",
            deal_id, result.deal_reference
        );
        Ok(result)
    }

    async fn close_position(
        &self,
        close_request: &ClosePositionRequest,
    ) -> Result<ClosePositionResponse, AppError> {
        info!("Closing position");

        // IG API requires POST with _method: DELETE header for closing positions
        // This is a workaround for HTTP client limitations with DELETE + body
        let result: ClosePositionResponse = self
            .http_client
            .post_with_delete_method("positions/otc", close_request, Some(1))
            .await?;

        debug!("Position closed with reference: {}", result.deal_reference);
        Ok(result)
    }

    async fn get_working_orders(&self) -> Result<WorkingOrdersResponse, AppError> {
        info!("Getting all working orders");
        let result: WorkingOrdersResponse = self.http_client.get("workingorders", Some(2)).await?;
        debug!("Retrieved {} working orders", result.working_orders.len());
        Ok(result)
    }

    async fn create_working_order(
        &self,
        order: &CreateWorkingOrderRequest,
    ) -> Result<CreateWorkingOrderResponse, AppError> {
        info!("Creating working order for: {}", order.epic);
        let result: CreateWorkingOrderResponse = self
            .http_client
            .post("workingorders/otc", order, Some(2))
            .await?;
        debug!(
            "Working order created with reference: {}",
            result.deal_reference
        );
        Ok(result)
    }
}
