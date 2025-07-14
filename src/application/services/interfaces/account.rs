use crate::application::models::account::{
    AccountActivity, AccountInfo, Positions, TransactionHistory, WorkingOrders,
};
use crate::error::AppError;
use crate::session::interface::IgSession;
use async_trait::async_trait;

/// Interface for the account service
#[async_trait]
pub trait AccountService: Send + Sync {
    /// Gets information about all user accounts
    async fn get_accounts(&self, session: &IgSession) -> Result<AccountInfo, AppError>;

    /// Gets open positions
    async fn get_positions(&self, session: &IgSession) -> Result<Positions, AppError>;

    /// Gets working orders
    async fn get_working_orders(&self, session: &IgSession) -> Result<WorkingOrders, AppError>;

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
        session: &IgSession,
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
        session: &IgSession,
        from: &str,
        to: &str,
    ) -> Result<AccountActivity, AppError>;

    /// Gets transaction history for a given period, handling pagination automatically.
    async fn get_transactions(
        &self,
        session: &IgSession,
        from: &str,
        to: &str,
    ) -> Result<TransactionHistory, AppError>;
}
