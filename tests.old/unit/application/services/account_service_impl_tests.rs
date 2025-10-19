use ig_client::application::services::account_service::AccountServiceImpl;
use ig_client::config::Config;
use ig_client::transport::http_client::IgHttpClientImpl;
use std::sync::Arc;

#[tokio::test]
async fn test_get_and_set_config() {
    let config = Arc::new(Config::new());
    let client = Arc::new(IgHttpClientImpl::new(config.clone()));
    let mut svc = AccountServiceImpl::new(config.clone(), client.clone());

    let cfg1 = svc.get_config();
    assert!(Arc::ptr_eq(&cfg1, &config));

    let new_cfg = Arc::new(Config::default());
    svc.set_config(new_cfg.clone());
    assert!(Arc::ptr_eq(&svc.get_config(), &new_cfg));
}
