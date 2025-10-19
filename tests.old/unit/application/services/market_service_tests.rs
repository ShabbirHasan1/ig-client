use ig_client::application::models::market::{
    HistoricalPricesResponse, MarketData, MarketNavigationNode, MarketNavigationResponse,
    MarketSearchResult,
};
use ig_client::application::services::MarketService;
use ig_client::application::services::market_service::MarketServiceImpl;
use ig_client::config::Config;
use ig_client::error::AppError;
use ig_client::presentation::InstrumentType;
use ig_client::session::interface::IgSession;
use ig_client::transport::http_client::IgHttpClient;
use ig_client::utils::rate_limiter::RateLimitType;
use reqwest::Method;
use serde::de::DeserializeOwned;
use std::sync::Arc;

// Mock HTTP client for testing service methods without actual network calls
struct MockHttpClient {}

#[async_trait::async_trait]
impl IgHttpClient for MockHttpClient {
    async fn request<T: serde::Serialize + Sync, R: DeserializeOwned>(
        &self,
        _method: Method,
        _path: &str,
        _session: &IgSession,
        _body: Option<&T>,
        _version: &str,
    ) -> Result<R, AppError> {
        // This mock will never be called in our tests
        // We're only testing validation logic that happens before network calls
        panic!("Mock HTTP client should not be called in these tests");
    }

    async fn request_no_auth<T: serde::Serialize + Send + Sync, R: DeserializeOwned>(
        &self,
        _method: Method,
        _path: &str,
        _body: Option<&T>,
        _version: &str,
    ) -> Result<R, AppError> {
        // This mock will never be called in our tests
        panic!("Mock HTTP client should not be called in these tests");
    }
}

#[test]
fn test_market_data_display() {
    // Create a MarketData instance
    let market_data = MarketData {
        epic: "OP.D.OTCDAX1.021100P.IP".to_string(),
        instrument_name: "EUR/USD".to_string(),
        instrument_type: InstrumentType::Currencies,
        expiry: "DFB".to_string(),
        high_limit_price: Some(1.2000),
        low_limit_price: Some(1.1000),
        market_status: "TRADEABLE".to_string(),
        net_change: Some(0.0010),
        percentage_change: Some(0.1),
        update_time: Some("22:00:00".to_string()),
        update_time_utc: None,
        bid: Some(1.1850),
        offer: Some(1.1852),
    };

    // Verify Display implementation works correctly
    let display_string = format!("{market_data}");
    assert!(display_string.contains("OP.D.OTCDAX1.021100P.IP"));
    assert!(display_string.contains("EUR/USD"));
    assert!(display_string.contains("TRADEABLE"));
}

#[test]
fn test_market_search_result() {
    // Create a MarketData instance
    let market_data = MarketData {
        epic: "OP.D.OTCDAX1.021100P.IP".to_string(),
        instrument_name: "EUR/USD".to_string(),
        instrument_type: InstrumentType::Currencies,
        expiry: "DFB".to_string(),
        high_limit_price: Some(1.2000),
        low_limit_price: Some(1.1000),
        market_status: "TRADEABLE".to_string(),
        net_change: Some(0.0010),
        percentage_change: Some(0.1),
        update_time: Some("22:00:00".to_string()),
        update_time_utc: None,
        bid: Some(1.1850),
        offer: Some(1.1852),
    };

    let search_result = MarketSearchResult {
        markets: vec![market_data],
    };

    // Verify structure was created correctly
    assert_eq!(search_result.markets.len(), 1);
    assert_eq!(search_result.markets[0].epic, "OP.D.OTCDAX1.021100P.IP");
    assert_eq!(search_result.markets[0].instrument_name, "EUR/USD");
    assert!(matches!(
        search_result.markets[0].instrument_type,
        InstrumentType::Currencies
    ));
}

#[test]
fn test_instrument_type() {
    // Verify that instrument types are defined correctly
    assert!(matches!(InstrumentType::Shares, InstrumentType::Shares));
    assert!(matches!(
        InstrumentType::Currencies,
        InstrumentType::Currencies
    ));
    assert!(matches!(InstrumentType::Indices, InstrumentType::Indices));
    assert!(matches!(
        InstrumentType::Commodities,
        InstrumentType::Commodities
    ));
    assert!(matches!(InstrumentType::Binary, InstrumentType::Binary));

    // Verify equality
    assert_eq!(InstrumentType::Shares, InstrumentType::Shares);
    assert_ne!(InstrumentType::Shares, InstrumentType::Currencies);
}

#[test]
fn test_market_service_config() {
    // Create a mock session
    let session = IgSession::new(
        "CST123".to_string(),
        "XST123".to_string(),
        "ACC123".to_string(),
    );

    // Verify session was created successfully
    assert_eq!(session.cst, "CST123");
    assert_eq!(session.token, "XST123");
    assert_eq!(session.account_id, "ACC123");

    // Test service config methods
    let config = Arc::new(Config::with_rate_limit_type(
        RateLimitType::NonTradingAccount,
        0.7,
    ));
    let client = Arc::new(MockHttpClient {});
    let mut service = MarketServiceImpl::new(config.clone(), client);

    // Test get_config
    assert!(std::ptr::eq(service.get_config(), &*config));

    // Test set_config
    let new_config = Arc::new(Config::default());
    service.set_config(new_config.clone());
    assert!(std::ptr::eq(service.get_config(), &*new_config));
}

#[test]
fn test_historical_prices_response() {
    // Create a PricePoint instance
    let price_point = ig_client::application::models::market::PricePoint {
        bid: Some(1.1850),
        ask: Some(1.1852),
        last_traded: Some(1.1851),
    };

    // Create a HistoricalPrice instance
    let historical_price = ig_client::application::models::market::HistoricalPrice {
        snapshot_time: "2023-01-01T00:00:00".to_string(),
        open_price: price_point.clone(),
        high_price: price_point.clone(),
        low_price: price_point.clone(),
        close_price: price_point,
        last_traded_volume: Some(1000),
    };

    // Create a PriceAllowance instance
    let price_allowance = ig_client::application::models::market::PriceAllowance {
        remaining_allowance: 9999,
        total_allowance: 10000,
        allowance_expiry: 3600,
    };

    // Create a HistoricalPricesResponse instance
    let historical_prices = HistoricalPricesResponse {
        prices: vec![historical_price],
        instrument_type: InstrumentType::Currencies,
        allowance: Some(price_allowance),
    };

    // Verify structure was created correctly
    assert_eq!(historical_prices.prices.len(), 1);
    assert_eq!(
        historical_prices.prices[0].snapshot_time,
        "2023-01-01T00:00:00"
    );
    assert_eq!(historical_prices.prices[0].open_price.bid, Some(1.1850));
    assert_eq!(historical_prices.prices[0].open_price.ask, Some(1.1852));
    assert_eq!(historical_prices.prices[0].last_traded_volume, Some(1000));
    assert!(matches!(
        historical_prices.instrument_type,
        InstrumentType::Currencies
    ));
    let allowance = &historical_prices.allowance.unwrap();
    assert_eq!(allowance.remaining_allowance, 9999);
    assert_eq!(allowance.total_allowance, 10000);
    assert_eq!(allowance.allowance_expiry, 3600);
}

#[tokio::test]
async fn test_get_multiple_market_details_empty_epics() {
    // Setup
    let config = Arc::new(Config::default());
    let client = Arc::new(MockHttpClient {});
    let service = MarketServiceImpl::new(config, client);
    let session = IgSession::new(
        "CST123".to_string(),
        "XST123".to_string(),
        "ACC123".to_string(),
    );

    // Test with empty epics array
    let empty_epics: Vec<String> = vec![];
    let result = service
        .get_multiple_market_details(&session, &empty_epics)
        .await;

    // Verify empty vector is returned without error
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_multiple_market_details_too_many_epics() {
    // Setup
    let config = Arc::new(Config::default());
    let client = Arc::new(MockHttpClient {});
    let service = MarketServiceImpl::new(config, client);
    let session = IgSession::new(
        "CST123".to_string(),
        "XST123".to_string(),
        "ACC123".to_string(),
    );

    // Create array with 51 epics (exceeding the 50 limit)
    let too_many_epics: Vec<String> = (0..51).map(|i| format!("EPIC{i}")).collect();

    // Test with too many epics
    let result = service
        .get_multiple_market_details(&session, &too_many_epics)
        .await;

    // Verify error is returned
    assert!(result.is_err());
    match result {
        Err(AppError::InvalidInput(msg)) => {
            assert!(msg.contains("maximum number of EPICs is 50"));
        }
        _ => panic!("Expected InvalidInput error"),
    }
}

#[test]
fn test_market_navigation_response() {
    // Create a market navigation response
    let nav_response = MarketNavigationResponse {
        nodes: vec![MarketNavigationNode {
            id: "123".to_string(),
            name: "Test Node".to_string(),
        }],
        markets: vec![MarketData {
            epic: "EPIC123".to_string(),
            instrument_name: "Test Market".to_string(),
            instrument_type: InstrumentType::Shares,
            expiry: "DFB".to_string(),
            high_limit_price: Some(100.0),
            low_limit_price: Some(90.0),
            market_status: "TRADEABLE".to_string(),
            net_change: Some(1.5),
            percentage_change: Some(1.5),
            update_time: Some("12:00:00".to_string()),
            update_time_utc: None,
            bid: Some(95.0),
            offer: Some(96.0),
        }],
    };

    // Verify structure
    assert_eq!(nav_response.nodes.len(), 1);
    assert_eq!(nav_response.nodes[0].id, "123");
    assert_eq!(nav_response.nodes[0].name, "Test Node");
    assert_eq!(nav_response.markets.len(), 1);
    assert_eq!(nav_response.markets[0].epic, "EPIC123");
}
