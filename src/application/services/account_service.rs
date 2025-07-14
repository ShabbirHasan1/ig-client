use crate::application::services::AccountService;
use crate::{
    application::models::account::{
        AccountActivity, AccountInfo, Positions, TransactionHistory, WorkingOrders,
    },
    config::Config,
    error::AppError,
    session::interface::IgSession,
    transport::http_client::IgHttpClient,
};
use async_trait::async_trait;
use reqwest::Method;
use std::sync::Arc;
use tracing::{debug, info};

/// Implementation of the account service
pub struct AccountServiceImpl<T: IgHttpClient> {
    config: Arc<Config>,
    client: Arc<T>,
}

impl<T: IgHttpClient> AccountServiceImpl<T> {
    /// Creates a new instance of the account service
    pub fn new(config: Arc<Config>, client: Arc<T>) -> Self {
        Self { config, client }
    }

    /// Gets the current configuration
    ///
    /// # Returns
    /// * The current configuration as an `Arc<Config>`
    pub fn get_config(&self) -> Arc<Config> {
        self.config.clone()
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
impl<T: IgHttpClient + 'static> AccountService for AccountServiceImpl<T> {
    async fn get_accounts(&self, session: &IgSession) -> Result<AccountInfo, AppError> {
        info!("Getting account information");

        let result = self
            .client
            .request::<(), AccountInfo>(Method::GET, "accounts", session, None, "1")
            .await?;

        debug!(
            "Account information obtained: {} accounts",
            result.accounts.len()
        );
        Ok(result)
    }

    async fn get_positions(&self, session: &IgSession) -> Result<Positions, AppError> {
        debug!("Getting open positions");

        let result = self
            .client
            .request::<(), Positions>(Method::GET, "positions", session, None, "2")
            .await?;

        debug!("Positions obtained: {} positions", result.positions.len());
        Ok(result)
    }

    async fn get_working_orders(&self, session: &IgSession) -> Result<WorkingOrders, AppError> {
        info!("Getting working orders");

        let result = self
            .client
            .request::<(), WorkingOrders>(Method::GET, "workingorders", session, None, "2")
            .await?;

        debug!(
            "Working orders obtained: {} orders",
            result.working_orders.len()
        );
        Ok(result)
    }

    async fn get_activity(
        &self,
        session: &IgSession,
        from: &str,
        to: &str,
    ) -> Result<AccountActivity, AppError> {
        let path = format!("history/activity?from={from}&to={to}&pageSize=500");
        info!("Getting account activity");

        let result = self
            .client
            .request::<(), AccountActivity>(Method::GET, &path, session, None, "3")
            .await?;

        debug!(
            "Account activity obtained: {} activities",
            result.activities.len()
        );
        Ok(result)
    }

    async fn get_activity_with_details(
        &self,
        session: &IgSession,
        from: &str,
        to: &str,
    ) -> Result<AccountActivity, AppError> {
        let path = format!("history/activity?from={from}&to={to}&detailed=true&pageSize=500");
        info!("Getting detailed account activity");

        let result = self
            .client
            .request::<(), AccountActivity>(Method::GET, &path, session, None, "3")
            .await?;

        debug!(
            "Detailed account activity obtained: {} activities",
            result.activities.len()
        );
        Ok(result)
    }

    async fn get_transactions(
        &self,
        session: &IgSession,
        from: &str,
        to: &str,
    ) -> Result<TransactionHistory, AppError> {
        const PAGE_SIZE: u32 = 200;
        let mut all_transactions = Vec::new();
        let mut current_page = 1;
        #[allow(unused_assignments)]
        let mut last_metadata = None;

        loop {
            let path = format!(
                "history/transactions?from={from}&to={to}&pageSize={PAGE_SIZE}&pageNumber={current_page}"
            );
            info!("Getting transaction history page {}", current_page);

            let result = self
                .client
                .request::<(), TransactionHistory>(Method::GET, &path, session, None, "2")
                .await?;

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

        Ok(TransactionHistory {
            transactions: all_transactions,
            metadata: last_metadata
                .ok_or_else(|| AppError::InvalidInput("Could not retrieve metadata".to_string()))?,
        })
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
            RateLimitType::TradingAccount,
            0.7,
        ));
        let client = Arc::new(IgHttpClientImpl::new(config.clone()));
        let mut service = AccountServiceImpl::new(config.clone(), client.clone());

        let cfg1 = service.get_config();
        assert!(Arc::ptr_eq(&cfg1, &config));

        let new_cfg = Arc::new(Config::default());
        service.set_config(new_cfg.clone());
        assert!(Arc::ptr_eq(&service.get_config(), &new_cfg));
    }
}
