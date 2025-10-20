
use crate::error::AppError;
use async_trait::async_trait;
use crate::model::auth::AccountInfo;
use crate::prelude::{AccountActivity, AccountsResponse};
use crate::presentation::account::{Positions, TransactionHistory, WorkingOrders};

/// Interface for the account service
#[async_trait]
pub trait AccountService: Send + Sync {
    /// Gets information about all user accounts
    async fn get_accounts(&self) -> Result<AccountsResponse, AppError>;

    /// Gets open positions
    async fn get_positions(&self) -> Result<Positions, AppError>;

    /// Gets open positions base in filter
    async fn get_positions_w_filter(
        &self,
        filter: &str,
    ) -> Result<Positions, AppError>;

    /// Gets working orders
    async fn get_working_orders(&self) -> Result<WorkingOrders, AppError>;

    /// Gets account activity
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `from` - Start date in ISO format (e.g. "2023-01-01T00:00:00Z")
    /// * `to` - End date in ISO format (e.g. "2023-02-01T00:00:00Z")
    ///
    /// # Returns
    /// * Account activity for the specified period
    async fn get_activity(
        &self,
        
        from: &str,
        to: &str,
    ) -> Result<AccountActivity, AppError>;

    /// Gets detailed account activity
    ///
    /// This method includes additional details for each activity item by using
    /// the detailed=true parameter in the API request.
    ///
    /// # Arguments
    /// * `session` - The current session
    /// * `from` - Start date in ISO format (e.g. "2023-01-01T00:00:00Z")
    /// * `to` - End date in ISO format (e.g. "2023-02-01T00:00:00Z")
    ///
    /// # Returns
    /// * Detailed account activity for the specified period
    async fn get_activity_with_details(
        &self,
        
        from: &str,
        to: &str,
    ) -> Result<AccountActivity, AppError>;

    /// Gets transaction history for a given period, handling pagination automatically.
    async fn get_transactions(
        &self,
        
        from: &str,
        to: &str,
    ) -> Result<TransactionHistory, AppError>;
}
