#[cfg(test)]
mod tests {
    use assert_json_diff::assert_json_eq;
    use ig_client::application::models::market::{
        Currency, DealingRules, HistoricalPricesResponse, Instrument, MarketData, MarketDetails,
        MarketSearchResult, PriceAllowance, PricePoint,
    };
    use ig_client::presentation::InstrumentType;
    use serde_json::json;

    #[test]
    fn test_instrument_type_serialization() {
        let types = vec![
            (InstrumentType::Shares, "SHARES"),
            (InstrumentType::Currencies, "CURRENCIES"),
            (InstrumentType::Indices, "INDICES"),
            (InstrumentType::SprintMarket, "SPRINT_MARKET"),
            (InstrumentType::Commodities, "COMMODITIES"),
            (InstrumentType::Options, "OPTIONS"),
            (InstrumentType::Binary, "BINARY"),
        ];

        for (instrument_type, expected_str) in types {
            let serialized = serde_json::to_string(&instrument_type).unwrap();
            assert_eq!(serialized, format!("\"{expected_str}\""));
        }
    }

    #[test]
    fn test_instrument_type_deserialization() {
        let types = vec![
            ("BINARY", InstrumentType::Binary),
            ("BUNGEE_CAPPED", InstrumentType::BungeeCapped),
            ("BUNGEE_COMMODITIES", InstrumentType::BungeeCommodities),
            ("BUNGEE_CURRENCIES", InstrumentType::BungeeCurrencies),
            ("BUNGEE_INDICES", InstrumentType::BungeeIndices),
            ("COMMODITIES", InstrumentType::Commodities),
            ("CURRENCIES", InstrumentType::Currencies),
            ("INDICES", InstrumentType::Indices),
            (
                "KNOCKOUTS_COMMODITIES",
                InstrumentType::KnockoutsCommodities,
            ),
            ("KNOCKOUTS_CURRENCIES", InstrumentType::KnockoutsCurrencies),
            ("KNOCKOUTS_INDICES", InstrumentType::KnockoutsIndices),
            ("KNOCKOUTS_SHARES", InstrumentType::KnockoutsShares),
            ("OPT_COMMODITIES", InstrumentType::OptCommodities),
            ("OPT_CURRENCIES", InstrumentType::OptCurrencies),
            ("OPT_INDICES", InstrumentType::OptIndices),
            ("OPT_RATES", InstrumentType::OptRates),
            ("OPT_SHARES", InstrumentType::OptShares),
            ("RATES", InstrumentType::Rates),
            ("SECTORS", InstrumentType::Sectors),
            ("SHARES", InstrumentType::Shares),
            ("SPRINT_MARKET", InstrumentType::SprintMarket),
            ("TEST_MARKET", InstrumentType::TestMarket),
            ("UNKNOWN", InstrumentType::Unknown),
            ("OPTIONS", InstrumentType::Options),
        ];

        for (json_str, expected_type) in types {
            let deserialized: InstrumentType =
                serde_json::from_str(&format!("\"{json_str}\"")).unwrap();
            assert_eq!(deserialized, expected_type);
        }
    }

    #[test]
    fn test_instrument_deserialization() {
        let json_str = r#"{
            "epic": "OP.D.OTCDAX1.021100P.IP",
            "name": "EUR/USD",
            "instrumentType": "CURRENCIES",
            "expiry": "DFB",
            "contractSize": "100000.0",
            "valueOfOnePip": "10.0",
            "lotSize": 1000.0,
            "highLimitPrice": 1.5,
            "lowLimitPrice": 0.5,
            "marginFactor": 3.33,
            "marginFactorUnit": "PERCENTAGE",
            "slippageFactor": { "unit": "POINTS", "value": 0.5 },
            "limitedRiskPremium": { "unit": "POINTS", "value": 0.1 },
            "newsCode": "eurusd",
            "chartCode": "EURUSD",
            "currencies": [
                {
                    "code": "USD",
                    "symbol": "$",
                    "baseExchangeRate": 1.0,
                    "exchangeRate": 1.0,
                    "isDefault": true
                }
            ]
        }"#;

        let instrument: Instrument = serde_json::from_str(json_str).unwrap();

        assert_eq!(instrument.epic, "OP.D.OTCDAX1.021100P.IP");
        assert_eq!(instrument.name, "EUR/USD");
        assert_eq!(instrument.instrument_type, Some(InstrumentType::Currencies));
        assert_eq!(instrument.expiry, "DFB");
        assert_eq!(instrument.contract_size, "100000.0");
        assert_eq!(instrument.value_of_one_pip, "10.0");
        assert_eq!(instrument.lot_size, Some(1000.0));
        assert_eq!(instrument.high_limit_price, Some(1.5));
        assert_eq!(instrument.low_limit_price, Some(0.5));
        assert_eq!(instrument.margin_factor, Some(3.33));
        assert_eq!(
            instrument.margin_factor_unit,
            Some("PERCENTAGE".to_string())
        );
        assert_eq!(
            instrument.slippage_factor.as_ref().unwrap().value,
            Some(0.5)
        );
        assert_eq!(
            instrument.limited_risk_premium.as_ref().unwrap().value,
            Some(0.1)
        );
        assert_eq!(instrument.news_code, Some("eurusd".to_string()));
        assert_eq!(instrument.chart_code, Some("EURUSD".to_string()));

        let currency = &instrument.currencies.as_ref().unwrap()[0];
        assert_eq!(currency.code, "USD");
        assert_eq!(currency.symbol, Some("$".to_string()));
        assert_eq!(currency.base_exchange_rate, Some(1.0));
        assert_eq!(currency.exchange_rate, Some(1.0));
        assert_eq!(currency.is_default, Some(true));
    }

    #[test]
    fn test_currency_deserialization() {
        let json_str = r#"{
            "code": "EUR",
            "symbol": "€",
            "baseExchangeRate": 1.1,
            "exchangeRate": 1.12,
            "isDefault": false
        }"#;

        let currency: Currency = serde_json::from_str(json_str).unwrap();

        assert_eq!(currency.code, "EUR");
        assert_eq!(currency.symbol, Some("€".to_string()));
        assert_eq!(currency.base_exchange_rate, Some(1.1));
        assert_eq!(currency.exchange_rate, Some(1.12));
        assert_eq!(currency.is_default, Some(false));
    }

    #[test]
    fn test_market_details_deserialization() {
        let json_str = r#"{
            "instrument": {
                "epic": "IX.D.FTSE.CFD.IP",
                "name": "FTSE 100",
                "instrumentType": "INDICES",
                "expiry": "DFB",
                "contractSize": "1.0",
                "valueOfOnePip": "10.0",
                "lotSize": 1.0,
                "highLimitPrice": 8000.0,
                "lowLimitPrice": 6000.0,
                "marginFactor": 5.0,
                "marginFactorUnit": "PERCENTAGE",
                "slippageFactor": { "unit": "POINTS", "value": 1.0 },
                "limitedRiskPremium": null,
                "newsCode": "FTSE100",
                "chartCode": "UKX",
                "currencies": []
            },
            "snapshot": {
                "marketStatus": "OPEN",
                "netChange": 15.5,
                "percentageChange": 0.25,
                "updateTime": "2023-05-13T10:15:30",
                "delayTime": 0,
                "bid": 7501.5,
                "offer": 7502.5,
                "high": 7520.0,
                "low": 7480.0,
                "binaryOdds": null,
                "decimalPlacesFactor": 1,
                "scalingFactor": 1,
                "controlledRiskExtraSpread": 0.5
            },
            "dealingRules": {
                "minStepDistance": { "unit": "POINTS", "value": 0.1 },
                "minDealSize": { "unit": "POINTS", "value": 0.1 },
                "maxDealSize": 1000.0,
                "minControlledRiskStopDistance": { "unit": "POINTS", "value": 5.0 },
                "minNormalStopOrLimitDistance": { "unit": "POINTS", "value": 2.0 },
                "maxStopOrLimitDistance": { "unit": "POINTS", "value": 2000.0 },
                "controlledRiskSpacing": { "unit": "POINTS", "value": 10.0 },
                "marketOrderPreference": "AVAILABLE_DEFAULT_ON",
                "trailingStopsPreference": "AVAILABLE"
            }
        }"#;

        let market_details: MarketDetails = serde_json::from_str(json_str).unwrap();

        assert_eq!(market_details.instrument.epic, "IX.D.FTSE.CFD.IP");
        assert_eq!(market_details.instrument.name, "FTSE 100");
        assert_eq!(
            market_details.instrument.instrument_type,
            Some(InstrumentType::Indices)
        );

        assert_eq!(market_details.snapshot.market_status, "OPEN");
        assert_eq!(market_details.snapshot.net_change, Some(15.5));
        assert_eq!(market_details.snapshot.bid, Some(7501.5));
        assert_eq!(market_details.snapshot.offer, Some(7502.5));

        assert_eq!(
            market_details.dealing_rules.market_order_preference,
            "AVAILABLE_DEFAULT_ON"
        );
        assert_eq!(
            market_details.dealing_rules.trailing_stops_preference,
            "AVAILABLE"
        );
    }

    #[test]
    fn test_dealing_rules_deserialization() {
        let json_str = r#"{
            "minStepDistance": { "unit": "POINTS", "value": 0.1 },
            "minDealSize": { "unit": "POINTS", "value": 0.1 },
            "maxDealSize": 1000.0,
            "minControlledRiskStopDistance": { "unit": "POINTS", "value": 5.0 },
            "minNormalStopOrLimitDistance": { "unit": "POINTS", "value": 2.0 },
            "maxStopOrLimitDistance": { "unit": "POINTS", "value": 2000.0 },
            "controlledRiskSpacing": { "unit": "POINTS", "value": 10.0 },
            "marketOrderPreference": "AVAILABLE_DEFAULT_ON",
            "trailingStopsPreference": "AVAILABLE"
        }"#;

        let dealing_rules: DealingRules = serde_json::from_str(json_str).unwrap();

        assert_eq!(dealing_rules.min_deal_size.value, Some(0.1));
        assert_eq!(dealing_rules.max_deal_size, Some(1000.0));
        assert_eq!(
            dealing_rules.min_controlled_risk_stop_distance.value,
            Some(5.0)
        );
        assert_eq!(
            dealing_rules.min_normal_stop_or_limit_distance.value,
            Some(2.0)
        );
        assert_eq!(dealing_rules.max_stop_or_limit_distance.value, Some(2000.0));
        assert_eq!(
            dealing_rules.market_order_preference,
            "AVAILABLE_DEFAULT_ON"
        );
        assert_eq!(dealing_rules.trailing_stops_preference, "AVAILABLE");
    }

    #[test]
    fn test_market_search_result_deserialization() {
        let json_str = r#"{
            "markets": [
                {
                    "epic": "OP.D.OTCDAX1.021100P.IP",
                    "instrumentName": "EUR/USD",
                    "instrumentType": "CURRENCIES",
                    "expiry": "DFB",
                    "highLimitPrice": 1.5,
                    "lowLimitPrice": 0.5,
                    "marketStatus": "OPEN",
                    "netChange": 0.0012,
                    "percentageChange": 0.1,
                    "updateTime": "2023-05-13T10:15:30",
                    "bid": 1.0876,
                    "offer": 1.0878
                },
                {
                    "epic": "CS.D.GBPUSD.CFD.IP",
                    "instrumentName": "GBP/USD",
                    "instrumentType": "CURRENCIES",
                    "expiry": "DFB",
                    "highLimitPrice": 1.6,
                    "lowLimitPrice": 0.6,
                    "marketStatus": "OPEN",
                    "netChange": -0.0023,
                    "percentageChange": -0.2,
                    "updateTime": "2023-05-13T10:15:30",
                    "bid": 1.2456,
                    "offer": 1.2458
                }
            ]
        }"#;

        let search_result: MarketSearchResult = serde_json::from_str(json_str).unwrap();

        assert_eq!(search_result.markets.len(), 2);
        assert_eq!(search_result.markets[0].epic, "OP.D.OTCDAX1.021100P.IP");
        assert_eq!(search_result.markets[0].instrument_name, "EUR/USD");
        assert_eq!(
            search_result.markets[0].instrument_type,
            InstrumentType::Currencies
        );
        assert_eq!(search_result.markets[0].bid, Some(1.0876));

        assert_eq!(search_result.markets[1].epic, "CS.D.GBPUSD.CFD.IP");
        assert_eq!(search_result.markets[1].instrument_name, "GBP/USD");
        assert_eq!(search_result.markets[1].net_change, Some(-0.0023));
    }

    #[test]
    fn test_market_data_display() {
        let market_data = MarketData {
            epic: "OP.D.OTCDAX1.021100P.IP".to_string(),
            instrument_name: "EUR/USD".to_string(),
            instrument_type: InstrumentType::Currencies,
            expiry: "DFB".to_string(),
            high_limit_price: Some(1.5),
            low_limit_price: Some(0.5),
            market_status: "OPEN".to_string(),
            net_change: Some(0.0012),
            percentage_change: Some(0.1),
            update_time: Some("2023-05-13T10:15:30".to_string()),
            update_time_utc: None,
            bid: Some(1.0876),
            offer: Some(1.0878),
        };

        let display_str = format!("{market_data}");
        let parsed_json: serde_json::Value = serde_json::from_str(&display_str).unwrap();

        let expected_json = json!({
            "epic": "OP.D.OTCDAX1.021100P.IP",
            "instrumentName": "EUR/USD",
            "instrumentType": "CURRENCIES",
            "expiry": "DFB",
            "highLimitPrice": 1.5,
            "lowLimitPrice": 0.5,
            "marketStatus": "OPEN",
            "netChange": 0.0012,
            "percentageChange": 0.1,
            "updateTime": "2023-05-13T10:15:30",
            "updateTimeUTC": null,
            "bid": 1.0876,
            "offer": 1.0878
        });

        assert_json_eq!(parsed_json, expected_json);
    }

    #[test]
    fn test_historical_prices_response_deserialization() {
        let json_str = r#"{
            "prices": [
                {
                    "snapshotTime": "2023-05-13T10:00:00",
                    "openPrice": {
                        "bid": 1.0870,
                        "ask": 1.0872,
                        "lastTraded": 1.0871
                    },
                    "highPrice": {
                        "bid": 1.0880,
                        "ask": 1.0882,
                        "lastTraded": 1.0881
                    },
                    "lowPrice": {
                        "bid": 1.0865,
                        "ask": 1.0867,
                        "lastTraded": 1.0866
                    },
                    "closePrice": {
                        "bid": 1.0876,
                        "ask": 1.0878,
                        "lastTraded": 1.0877
                    },
                    "lastTradedVolume": 25000
                }
            ],
            "instrumentType": "CURRENCIES",
            "allowance": {
                "remainingAllowance": 9995,
                "totalAllowance": 10000,
                "allowanceExpiry": 3600
            }
        }"#;

        let prices_response: HistoricalPricesResponse = serde_json::from_str(json_str).unwrap();

        assert_eq!(prices_response.instrument_type, InstrumentType::Currencies);
        assert_eq!(prices_response.prices.len(), 1);
        assert_eq!(
            prices_response.prices[0].snapshot_time,
            "2023-05-13T10:00:00"
        );
        assert_eq!(prices_response.prices[0].open_price.bid, Some(1.0870));
        assert_eq!(prices_response.prices[0].open_price.ask, Some(1.0872));
        assert_eq!(
            prices_response.prices[0].open_price.last_traded,
            Some(1.0871)
        );
        assert_eq!(prices_response.prices[0].last_traded_volume, Some(25000));
        let allowance = &prices_response.allowance.unwrap();
        assert_eq!(allowance.remaining_allowance, 9995);
        assert_eq!(allowance.total_allowance, 10000);
        assert_eq!(allowance.allowance_expiry, 3600);
    }

    #[test]
    fn test_price_point_deserialization() {
        let json_str = r#"{
            "bid": 7501.5,
            "ask": 7502.5,
            "lastTraded": 7502.0
        }"#;

        let price_point: PricePoint = serde_json::from_str(json_str).unwrap();

        assert_eq!(price_point.bid, Some(7501.5));
        assert_eq!(price_point.ask, Some(7502.5));
        assert_eq!(price_point.last_traded, Some(7502.0));
    }

    #[test]
    fn test_price_allowance_deserialization() {
        let json_str = r#"{
            "remainingAllowance": 9990,
            "totalAllowance": 10000,
            "allowanceExpiry": 3600
        }"#;

        let allowance: PriceAllowance = serde_json::from_str(json_str).unwrap();

        assert_eq!(allowance.remaining_allowance, 9990);
        assert_eq!(allowance.total_allowance, 10000);
        assert_eq!(allowance.allowance_expiry, 3600);
    }
}
