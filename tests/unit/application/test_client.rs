use ig_client::application::client::Client;
use ig_client::error::AppError;
use ig_client::application::interfaces::market::MarketService;

#[tokio::test]
async fn get_multiple_market_details_empty_returns_default() {
    let client = Client::new();
    let resp = client.get_multiple_market_details(&[]).await.expect("should be Ok for empty");
    assert!(resp.market_details.is_empty());
}

#[tokio::test]
async fn get_multiple_market_details_more_than_50_returns_error() {
    let client = Client::new();
    // Build 51 dummy EPICs
    let epics: Vec<String> = (0..51).map(|i| format!("EPIC{}", i)).collect();
    let err = client.get_multiple_market_details(&epics).await.err().expect("should be Err");
    match err {
        AppError::InvalidInput(msg) => {
            assert!(msg.contains("maximum number of EPICs is 50"));
        }
        other => panic!("Unexpected error: {:?}", other),
    }
}

#[test]
fn client_default_new_equivalence() {
    let _c1 = Client::new();
    let _c2: Client = Default::default();
    // Construction should not panic; no further assertions needed
}
