use async_trait::async_trait;
use ig_client::application::models::account::{
    Account, AccountActivity, AccountBalance, AccountInfo, PageData, Positions, TransactionHistory,
    TransactionMetadata, WorkingOrders,
};
use ig_client::application::services::AccountService;
use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::config::Config;
use ig_client::error::AppError;
use ig_client::session::interface::IgSession;
use ig_client::transport::http_client::IgHttpClient;
use reqwest::Method;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

// Mock HTTP client for testing service methods without actual network calls
struct MockHttpClient {
    expected_path: String,
    response_type: ResponseType,
}

enum ResponseType {
    AccountInfo,
    Positions,
    WorkingOrders,
    AccountActivity,
    TransactionHistory,
}

impl MockHttpClient {
    fn new(expected_path: &str, response_type: ResponseType) -> Self {
        Self {
            expected_path: expected_path.to_string(),
            response_type,
        }
    }

    fn create_response<R: DeserializeOwned>(&self) -> Result<R, AppError> {
        // Instead of serializing/deserializing, we'll create mock responses directly
        // This avoids the need for Serialize trait on the response types
        match self.response_type {
            ResponseType::AccountInfo => {
                // For AccountInfo, we'll create a JSON string directly and deserialize it
                let json = r#"{
                    "accounts": [{
                        "accountId": "TEST-123",
                        "accountName": "Test Account",
                        "currency": "USD",
                        "balance": {
                            "balance": 1000.0,
                            "deposit": 1000.0,
                            "profitLoss": 0.0,
                            "available": 1000.0
                        },
                        "accountType": "CFD",
                        "preferred": true,
                        "status": "ENABLED"
                    }]
                }"#;
                serde_json::from_str(json).map_err(|e| AppError::SerializationError(e.to_string()))
            }
            ResponseType::Positions => {
                // For Positions, create a JSON string directly
                let json = r#"{
                    "positions": []
                }"#;
                serde_json::from_str(json).map_err(|e| AppError::SerializationError(e.to_string()))
            }
            ResponseType::WorkingOrders => {
                // For WorkingOrders, create a JSON string directly
                let json = r#"{
                    "workingOrders": []
                }"#;
                serde_json::from_str(json).map_err(|e| AppError::SerializationError(e.to_string()))
            }
            ResponseType::AccountActivity => {
                // For AccountActivity, create a JSON string directly
                let json = r#"{
                    "activities": [],
                    "metadata": null
                }"#;
                serde_json::from_str(json).map_err(|e| AppError::SerializationError(e.to_string()))
            }
            ResponseType::TransactionHistory => {
                // For TransactionHistory, create a JSON string directly
                let json = r#"{
                    "transactions": [],
                    "metadata": {
                        "pageData": {
                            "pageSize": 10,
                            "pageNumber": 1,
                            "totalPages": 1
                        },
                        "size": 0
                    }
                }"#;
                serde_json::from_str(json).map_err(|e| AppError::SerializationError(e.to_string()))
            }
        }
    }
}

#[async_trait]
impl IgHttpClient for MockHttpClient {
    async fn request<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        _method: Method,
        path: &str,
        _session: &IgSession,
        _body: Option<&T>,
        _version: &str,
    ) -> Result<R, AppError> {
        // Verify the path matches what we expect
        assert_eq!(
            path, self.expected_path,
            "Path mismatch: expected '{}', got '{}'",
            self.expected_path, path
        );

        // Return a mock response based on the response type
        self.create_response()
    }

    async fn request_no_auth<T: Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        _method: Method,
        _path: &str,
        _body: Option<&T>,
        _version: &str,
    ) -> Result<R, AppError> {
        panic!("request_no_auth should not be called in these tests");
    }
}

#[test]
fn test_account_info_structure() {
    // Test the AccountInfo structure with a sample account
    let account_info = AccountInfo {
        accounts: vec![Account {
            account_id: "ACCOUNT-123".to_string(),
            account_name: "Test Account".to_string(),
            currency: "EUR".to_string(),
            balance: AccountBalance {
                balance: 1000.0,
                deposit: 1000.0,
                profit_loss: 0.0,
                available: 1000.0,
            },
            account_type: "CFD".to_string(),
            preferred: true,
            status: "ENABLED".to_string(),
        }],
    };

    assert_eq!(account_info.accounts.len(), 1);
    assert_eq!(account_info.accounts[0].account_id, "ACCOUNT-123");
    assert_eq!(account_info.accounts[0].account_name, "Test Account");
    assert_eq!(account_info.accounts[0].currency, "EUR");
    assert_eq!(account_info.accounts[0].balance.balance, 1000.0);
    assert_eq!(account_info.accounts[0].account_type, "CFD");
    assert!(account_info.accounts[0].preferred);
    assert_eq!(account_info.accounts[0].status, "ENABLED");
}

#[test]
fn test_positions_structure() {
    // Test the Positions structure
    let positions = Positions {
        positions: Vec::new(),
    };

    assert!(positions.positions.is_empty());
}

#[test]
fn test_working_orders_structure() {
    // Test the WorkingOrders structure
    let working_orders = WorkingOrders {
        working_orders: Vec::new(),
    };

    assert!(working_orders.working_orders.is_empty());
}

#[test]
fn test_account_activity_structure() {
    // Test the AccountActivity structure
    let activity = AccountActivity {
        activities: Vec::new(),
        metadata: None,
    };

    assert!(activity.activities.is_empty());
}

#[test]
fn test_transaction_history_structure() {
    // Test the TransactionHistory structure
    let history = TransactionHistory {
        transactions: Vec::new(),
        metadata: TransactionMetadata {
            page_data: PageData {
                page_size: 10,
                page_number: 1,
                total_pages: 1,
            },
            size: 0,
        },
    };

    assert!(history.transactions.is_empty());
    assert_eq!(history.metadata.page_data.page_size, 10);
    assert_eq!(history.metadata.page_data.page_number, 1);
    assert_eq!(history.metadata.page_data.total_pages, 1);
    assert_eq!(history.metadata.size, 0);
}

#[tokio::test]
async fn test_account_service_get_accounts() {
    // Create a mock client that expects the correct path
    let mock_client = Arc::new(MockHttpClient::new("accounts", ResponseType::AccountInfo));
    let config = Arc::new(Config::default());
    let service = AccountServiceImpl::new(config, mock_client);

    // Create a mock session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Call the method - this will verify the path is correct
    let result = service.get_accounts(&session).await;
    assert!(result.is_ok());

    // Verify the response structure
    let accounts = result.unwrap();
    assert_eq!(accounts.accounts.len(), 1);
    assert_eq!(accounts.accounts[0].account_id, "TEST-123");
}

#[tokio::test]
async fn test_account_service_get_positions() {
    // Create a mock client that expects the correct path
    let mock_client = Arc::new(MockHttpClient::new("positions", ResponseType::Positions));
    let config = Arc::new(Config::default());
    let service = AccountServiceImpl::new(config, mock_client);

    // Create a mock session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Call the method - this will verify the path is correct
    let result = service.get_positions(&session).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_account_service_get_working_orders() {
    // Create a mock client that expects the correct path
    let mock_client = Arc::new(MockHttpClient::new(
        "workingorders",
        ResponseType::WorkingOrders,
    ));
    let config = Arc::new(Config::default());
    let service = AccountServiceImpl::new(config, mock_client);

    // Create a mock session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Call the method - this will verify the path is correct
    let result = service.get_working_orders(&session).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_account_service_get_activity() {
    // Create a mock client that expects the correct path with query parameters
    let expected_path = "history/activity?from=2023-01-01&to=2023-01-31&pageSize=500";
    let mock_client = Arc::new(MockHttpClient::new(
        expected_path,
        ResponseType::AccountActivity,
    ));
    let config = Arc::new(Config::default());
    let service = AccountServiceImpl::new(config, mock_client);

    // Create a mock session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Call the method - this will verify the path is correct
    let result = service
        .get_activity(&session, "2023-01-01", "2023-01-31")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_account_service_get_activity_with_details() {
    // Create a mock client that expects the correct path with query parameters
    let expected_path = "history/activity?from=2023-01-01&to=2023-01-31&detailed=true&pageSize=500";
    let mock_client = Arc::new(MockHttpClient::new(
        expected_path,
        ResponseType::AccountActivity,
    ));
    let config = Arc::new(Config::default());
    let service = AccountServiceImpl::new(config, mock_client);

    // Create a mock session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Call the method - this will verify the path is correct
    let result = service
        .get_activity_with_details(&session, "2023-01-01", "2023-01-31")
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_account_service_get_transactions() {
    // Create a mock client that expects the correct path with query parameters
    let expected_path =
        "history/transactions?from=2023-01-01&to=2023-01-31&pageSize=200&pageNumber=1";
    let mock_client = Arc::new(MockHttpClient::new(
        expected_path,
        ResponseType::TransactionHistory,
    ));
    let config = Arc::new(Config::default());
    let service = AccountServiceImpl::new(config, mock_client);

    // Create a mock session
    let session = IgSession::new(
        "test_cst".to_string(),
        "test_token".to_string(),
        "test_account".to_string(),
    );

    // Call the method - this will verify the path is correct
    let result = service
        .get_transactions(&session, "2023-01-01", "2023-01-31")
        .await;
    assert!(result.is_ok());
}
