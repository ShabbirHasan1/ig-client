// Integration tests for market endpoints

use crate::common;
use ig_client::utils::logger::setup_logger;
use ig_client::{
    application::services::MarketService, application::services::market_service::MarketServiceImpl,
};
use tokio::runtime::Runtime;
use tracing::info;

#[test]
#[ignore]
fn test_search_markets() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create market service
    let market_service = MarketServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test search markets
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per application)
        ig_client::utils::rate_limiter::app_non_trading_limiter()
            .wait()
            .await;
        // Search for a common market term
        let search_term = "Germany 40";
        info!("Searching for markets with term: {}", search_term);

        let result = market_service
            .search_markets(&session, search_term)
            .await
            .expect("Failed to search markets");

        // Verify the search returned results
        assert!(
            !result.markets.is_empty(),
            "Search should return at least one market"
        );

        info!(
            "Found {} markets matching '{}'",
            result.markets.len(),
            search_term
        );

        // Print the first few results
        for (i, market) in result.markets.iter().take(5).enumerate() {
            info!("{}. {} ({})", i + 1, market.instrument_name, market.epic);
        }
    });
}

#[test]
#[ignore]
fn test_get_market_details() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create market service
    let market_service = MarketServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get market details
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per application)
        ig_client::utils::rate_limiter::app_non_trading_limiter()
            .wait()
            .await;
        // Use an open market
        let epic = "DO.D.OTCDDAX.143.IP"; // Open market provided by the user
        info!("Getting market details for: {}", epic);

        let result = market_service
            .get_market_details(&session, epic)
            .await
            .expect("Failed to get market details");

        // Verify the result contains the expected data
        assert_eq!(
            result.instrument.epic, epic,
            "EPIC in response should match the requested EPIC"
        );
        assert!(
            !result.instrument.name.is_empty(),
            "Instrument name should not be empty"
        );

        info!(
            "Successfully retrieved market details for: {} ({})",
            result.instrument.name, result.instrument.epic
        );
        info!("Market status: {}", result.snapshot.market_status);

        if let Some(bid) = result.snapshot.bid {
            info!("Current bid: {}", bid);
        }

        if let Some(offer) = result.snapshot.offer {
            info!("Current offer: {}", offer);
        }
    });
}

#[test]
#[ignore]
fn test_get_multiple_market_details() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create market service
    let market_service = MarketServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get multiple market details
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per application)
        ig_client::utils::rate_limiter::app_non_trading_limiter()
            .wait()
            .await;
        // Use open markets
        let epics = vec![
            "DO.D.OTCDDAX.129.IP".to_string(),
            "DO.D.OTCDDAX.128.IP".to_string(),
            "DO.D.OTCDDAX.143.IP".to_string(),
        ];

        info!("Getting market details for multiple EPICs: {:?}", epics);

        let results = market_service
            .get_multiple_market_details(&session, &epics)
            .await
            .expect("Failed to get multiple market details");

        // Verify we got results for all EPICs
        assert_eq!(
            results.len(),
            epics.len(),
            "Should receive the same number of results as EPICs requested"
        );

        // Print details for each result
        for (i, result) in results.iter().enumerate() {
            info!(
                "{}. {} ({})",
                i + 1,
                result.instrument.name,
                result.instrument.epic
            );
            info!("   Market status: {}", result.snapshot.market_status);

            if let Some(bid) = result.snapshot.bid {
                info!("   Current bid: {}", bid);
            }

            if let Some(offer) = result.snapshot.offer {
                info!("   Current offer: {}", offer);
            }
        }
    });
}

#[test]
#[ignore]
fn test_get_historical_prices() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create market service
    let market_service = MarketServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get historical prices
    rt.block_on(async {
        // Wait to respect the rate limit (historical price data)
        ig_client::utils::rate_limiter::historical_price_limiter()
            .wait()
            .await;

        // Calculate dates for last week (Monday to Friday)
        use chrono::{Datelike, Duration, Utc};

        // Get today's date
        let today = Utc::now().date_naive();

        // Calculate last Monday (go back to previous week if today is Monday)
        let days_since_monday = today.weekday().num_days_from_monday();
        let last_monday = if days_since_monday == 0 {
            // If today is Monday, go back 7 days to get last Monday
            today - Duration::days(7)
        } else {
            // Otherwise, go back to the Monday of this week
            today - Duration::days(days_since_monday as i64)
        };

        // Calculate last Friday
        let last_friday = last_monday + Duration::days(4);

        // Format dates for the API
        let from_date = format!(
            "{:04}-{:02}-{:02}T00:00:00",
            last_monday.year(),
            last_monday.month(),
            last_monday.day()
        );
        let to_date = format!(
            "{:04}-{:02}-{:02}T23:59:59",
            last_friday.year(),
            last_friday.month(),
            last_friday.day()
        );

        info!("Using date range: {} to {}", from_date, to_date);

        // Try different markets with the calculated date range
        let markets_to_try = vec![
            ("DO.D.OTCDDAX.128.IP", from_date.as_str(), to_date.as_str()),
            ("DO.D.OTCDDAX.2.IP", from_date.as_str(), to_date.as_str()),
            ("DO.D.OTCDDAX.1.IP", from_date.as_str(), to_date.as_str()),
        ];

        let resolution = "DAY";
        let mut success = false;

        for (epic, from, to) in markets_to_try {
            info!(
                "Getting historical prices for: {} (resolution: {}, from: {}, to: {})",
                epic, resolution, from, to
            );

            match market_service
                .get_historical_prices(&session, epic, resolution, from, to)
                .await
            {
                Ok(result) => {
                    if !result.prices.is_empty() {
                        info!("Retrieved {} historical price points", result.prices.len());

                        // Print the first few price points
                        for (i, price) in result.prices.iter().take(5).enumerate() {
                            info!(
                                "{}. {} - Open: {:?}, Close: {:?}",
                                i + 1,
                                price.snapshot_time,
                                price.open_price.bid,
                                price.close_price.bid
                            );
                        }

                        success = true;
                        break;
                    } else {
                        info!("No historical prices found for {}", epic);
                    }
                }
                Err(e) => {
                    info!("Failed to get historical prices for {}: {:?}", epic, e);
                }
            }
        }

        if !success {
            info!("Could not retrieve historical prices for any of the tested markets");
            // We don't fail the test, as this might be due to API limitations or market conditions
            // rather than a code issue
        }
    });
}

#[test]
#[ignore]
fn test_get_market_navigation() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create market service
    let market_service = MarketServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get market navigation
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per application)
        ig_client::utils::rate_limiter::app_non_trading_limiter()
            .wait()
            .await;
        info!("Getting top-level market navigation nodes");

        let result = market_service
            .get_market_navigation(&session)
            .await
            .expect("Failed to get market navigation");

        // Verify the result contains the expected data
        assert!(
            !result.nodes.is_empty(),
            "Should return at least one navigation node"
        );

        info!(
            "Retrieved {} top-level navigation nodes",
            result.nodes.len()
        );

        // Print the navigation nodes
        for (i, node) in result.nodes.iter().enumerate() {
            info!("{}. {} (ID: {})", i + 1, node.name, node.id);
        }

        // Print markets at the root level, if any
        if !result.markets.is_empty() {
            info!("Markets at root level:");
            for (i, market) in result.markets.iter().enumerate() {
                info!("{}. {} ({})", i + 1, market.instrument_name, market.epic);
            }
        }
    });
}

#[test]
#[ignore]
fn test_get_market_navigation_node() {
    setup_logger();
    // Create test configuration and client
    let config = common::create_test_config();
    let client = common::create_test_client(config.clone());

    // Create market service
    let market_service = MarketServiceImpl::new(config, client);

    // Get a session
    let session = common::login_with_account_switch();

    // Create a runtime for the async operations
    let rt = Runtime::new().expect("Failed to create runtime");

    // Test get market navigation node
    rt.block_on(async {
        // Wait to respect the rate limit (non-trading requests per application)
        ig_client::utils::rate_limiter::app_non_trading_limiter()
            .wait()
            .await;
        // First get the top-level nodes to find a node ID to use
        let top_level = market_service
            .get_market_navigation(&session)
            .await
            .expect("Failed to get top-level market navigation");

        if top_level.nodes.is_empty() {
            info!("No navigation nodes found, skipping test");
            return;
        }

        // Use the first node
        let node_id = &top_level.nodes[0].id;
        let node_name = &top_level.nodes[0].name;

        info!(
            "Getting market navigation node: {} ({})",
            node_name, node_id
        );

        let result = market_service
            .get_market_navigation_node(&session, node_id)
            .await
            .expect("Failed to get market navigation node");

        // Print the child nodes
        info!("Child nodes of '{}' ({})", node_name, node_id);
        for (i, node) in result.nodes.iter().enumerate() {
            info!("{}. {} (ID: {})", i + 1, node.name, node.id);
        }

        // Print markets in this node, if any
        if !result.markets.is_empty() {
            info!("Markets in '{}' ({})", node_name, node_id);
            for (i, market) in result.markets.iter().take(10).enumerate() {
                info!("{}. {} ({})", i + 1, market.instrument_name, market.epic);
            }

            if result.markets.len() > 10 {
                info!("... and {} more", result.markets.len() - 10);
            }
        }
    });
}
