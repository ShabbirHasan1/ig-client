use std::error::Error;
use std::matches;
use crate::application::models::market::{HistoricalPricesResponse, MarketDetails, MarketNavigationResponse, MarketNode, MarketSearchResult};
use crate::error::AppError;
use crate::session::interface::IgSession;
use async_trait::async_trait;
use tracing::{error, info};
use crate::application::services::types::DBEntry;
use crate::presentation::build_market_hierarchy;

/// Interface for the market service
#[async_trait]
pub trait MarketService: Send + Sync {
    /// Searches markets by search term
    async fn search_markets(
        &self,
        session: &IgSession,
        search_term: &str,
    ) -> Result<MarketSearchResult, AppError>;

    /// Gets details of a specific market by its EPIC
    async fn get_market_details(
        &self,
        session: &IgSession,
        epic: &str,
    ) -> Result<MarketDetails, AppError>;

    /// Gets details of multiple markets by their EPICs in a single request
    ///
    /// This method accepts a vector of EPICs and returns a vector of market details.
    /// The EPICs are sent as a comma-separated list in a single API request.
    ///
    /// # Arguments
    /// * `session` - The active IG session
    /// * `epics` - A slice of EPICs to get details for
    ///
    /// # Returns
    /// A vector of market details in the same order as the input EPICs
    async fn get_multiple_market_details(
        &self,
        session: &IgSession,
        epics: &[String],
    ) -> Result<Vec<MarketDetails>, AppError>;

    /// Gets historical prices for a market
    async fn get_historical_prices(
        &self,
        session: &IgSession,
        epic: &str,
        resolution: &str,
        from: &str,
        to: &str,
    ) -> Result<HistoricalPricesResponse, AppError>;

    /// Gets the top-level market navigation nodes
    ///
    /// This method returns the root nodes of the market hierarchy, which can be used
    /// to navigate through the available markets.
    async fn get_market_navigation(
        &self,
        session: &IgSession,
    ) -> Result<MarketNavigationResponse, AppError>;

    /// Gets the market navigation node with the specified ID
    ///
    /// This method returns the child nodes and markets under the specified node ID.
    ///
    /// # Arguments
    /// * `node_id` - The ID of the navigation node to retrieve
    async fn get_market_navigation_node(
        &self,
        session: &IgSession,
        node_id: &str,
    ) -> Result<MarketNavigationResponse, AppError>;
    async fn get_vec_db_entries(&self, session: &IgSession) -> Result<Vec<DBEntry>, AppError>;
}
