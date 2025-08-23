use ig_client::application::models::account::AccountTransaction;
use ig_client::application::models::transaction::{StoreTransaction, TransactionList};

// Sample JSON for a simple transaction
fn sample_raw_transaction_json() -> &'static str {
    "{\"date\":\"2023-01-01T10:00:00\",\"dateUtc\":\"2023-01-01T09:00:00\",\"openDateUtc\":\"2023-01-01T08:00:00\",\"instrumentName\":\"OP.D.OTCDAX1.021100P.IP\",\"period\":\"DFB\",\"profitAndLoss\":\"10.50\",\"transactionType\":\"DEAL\",\"reference\":\"REF123\",\"openLevel\":\"1.1000\",\"closeLevel\":\"1.1010\",\"size\":\"1.0\",\"currency\":\"EUR\",\"cashTransaction\":false}"
}

// Sample JSON with matching dateUtc equal to date
fn sample_raw_transaction_json2() -> &'static str {
    "{\"date\":\"2023-01-01T10:00:00\",\"dateUtc\":\"2023-01-01T10:00:00\",\"openDateUtc\":\"2023-01-01T09:00:00\",\"instrumentName\":\"OP.D.OTCDAX1.021100P.IP\",\"period\":\"DFB\",\"profitAndLoss\":\"10.50\",\"transactionType\":\"DEAL\",\"reference\":\"REF123\",\"openLevel\":\"1.1000\",\"closeLevel\":\"1.1010\",\"size\":\"1.0\",\"currency\":\"EUR\",\"cashTransaction\":false}"
}

// Sample JSON with negative P&L and different instrument
fn sample_raw_transaction_json3() -> &'static str {
    "{\"date\":\"2023-01-01T10:00:00\",\"dateUtc\":\"2023-01-01T10:00:00\",\"openDateUtc\":\"2023-01-01T09:00:00\",\"instrumentName\":\"CS.D.AAPL.CFD.IP\",\"period\":\"DFB\",\"profitAndLoss\":\"-5.25\",\"transactionType\":\"DEAL\",\"reference\":\"OPT123\",\"openLevel\":\"150.00\",\"closeLevel\":\"145.00\",\"size\":\"1.0\",\"currency\":\"USD\",\"cashTransaction\":false}"
}

#[test]
fn test_raw_transaction_display_and_serialization() {
    let json = sample_raw_transaction_json();
    // Deserialize from JSON
    let raw_tx: AccountTransaction = serde_json::from_str(json).expect("Failed to parse JSON");
    // Display implementation should produce valid JSON
    let display_str = raw_tx.to_string();
    assert!(display_str.contains("\"instrumentName\":\"OP.D.OTCDAX1.021100P.IP\""));
    assert!(display_str.contains("\"transactionType\":\"DEAL\""));
    // Round-trip serialization
    let serialized = serde_json::to_string(&raw_tx).expect("Failed to serialize RawTransaction");
    assert!(serialized.contains("\"reference\":\"REF123\""));
    // Deserialize again and compare display outputs
    let raw_tx2: AccountTransaction =
        serde_json::from_str(&serialized).expect("Failed to parse serialized JSON");
    assert_eq!(display_str, raw_tx2.to_string());
}

#[test]
fn test_transaction_creation() {
    let json = sample_raw_transaction_json2();
    // Deserialize from JSON
    let raw_tx: AccountTransaction = serde_json::from_str(json).expect("Failed to parse JSON");
    // Test the display implementation
    let display_str = raw_tx.to_string();
    assert!(display_str.contains("\"instrumentName\":\"OP.D.OTCDAX1.021100P.IP\""));
    assert!(display_str.contains("\"transactionType\":\"DEAL\""));
    assert!(display_str.contains("\"reference\":\"REF123\""));
}

#[test]
fn test_raw_transaction_serialization() {
    let json = sample_raw_transaction_json3();
    // Deserialize from JSON
    let raw_tx: AccountTransaction = serde_json::from_str(json).expect("Failed to parse JSON");
    // Test serialization to JSON
    let serialized = serde_json::to_string(&raw_tx).expect("Failed to serialize RawTransaction");
    assert!(serialized.contains("\"instrumentName\":\"CS.D.AAPL.CFD.IP\""));
    assert!(serialized.contains("\"transactionType\":\"DEAL\""));
    assert!(serialized.contains("\"reference\":\"OPT123\""));
    // Test deserialization back to RawTransaction
    let deserialized: AccountTransaction =
        serde_json::from_str(&serialized).expect("Failed to parse serialized JSON");
    // Convert back to JSON to verify the data was preserved
    let serialized_again = serde_json::to_string(&deserialized)
        .expect("Failed to serialize deserialized RawTransaction");
    assert_eq!(serialized, serialized_again);
}

#[test]
fn test_store_transaction_from_account_transaction() {
    // Create a mock AccountTransaction
    let account_tx = AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "2025-05-15T09:00:00".to_string(),
        instrument_name: "Daily Gold (Aug Future) 3395 CALL".to_string(),
        period: "MAY-25".to_string(),
        profit_and_loss: "E100.50".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "ABCD1234".to_string(),
        open_level: "1950.5".to_string(),
        close_level: "2050.75".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    // Convert to StoreTransaction
    let store_tx = StoreTransaction::from(account_tx);

    // Now we can directly access the fields since they're public
    assert_eq!(store_tx.transaction_type, "DEAL");
    assert_eq!(store_tx.reference, "ABCD1234");
    assert_eq!(store_tx.pnl_eur, 100.50);
    assert!(!store_tx.is_fee);
    assert_eq!(store_tx.underlying, Some("Gold".to_string()));
    assert_eq!(store_tx.option_type, Some("CALL".to_string()));
    assert_eq!(store_tx.strike, Some(3395.0));
    assert!(store_tx.expiry.is_some());
}

#[test]
fn test_store_transaction_from_account_transaction_with_fee() {
    // Create a mock fee transaction
    let fee_tx = AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "".to_string(),
        instrument_name: "Daily Admin Fee".to_string(),
        period: "".to_string(),
        profit_and_loss: "E-0.50".to_string(),
        transaction_type: "WITH".to_string(),
        reference: "FEE1234".to_string(),
        open_level: "0".to_string(),
        close_level: "0".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: true,
    };

    // Convert to StoreTransaction
    let store_tx = StoreTransaction::from(fee_tx);

    // Now we can directly access the fields
    assert_eq!(store_tx.transaction_type, "WITH");
    assert_eq!(store_tx.reference, "FEE1234");
    assert_eq!(store_tx.pnl_eur, -0.50);
    assert!(store_tx.is_fee);
    assert_eq!(store_tx.underlying, Some("Admin Fee".to_string()));
    assert_eq!(store_tx.option_type, None);
    assert_eq!(store_tx.strike, None);
    assert_eq!(store_tx.expiry, None);
}

#[test]
fn test_store_transaction_from_account_transaction_with_specific_date() {
    // Create a mock AccountTransaction with a specific date format
    let account_tx = AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "2025-05-15T09:00:00".to_string(),
        instrument_name: "Weekly US 500 (Wed) 5200 CALL ($1)".to_string(),
        period: "15-MAY-25".to_string(), // Specific date format
        profit_and_loss: "E-75.25".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "EFGH5678".to_string(),
        open_level: "4600".to_string(),
        close_level: "4450".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    // Convert to StoreTransaction
    let store_tx = StoreTransaction::from(account_tx);

    // Now we can directly access the fields
    assert_eq!(store_tx.transaction_type, "DEAL");
    assert_eq!(store_tx.reference, "EFGH5678");
    assert_eq!(store_tx.pnl_eur, -75.25);
    assert!(!store_tx.is_fee);
    assert_eq!(store_tx.underlying, Some("US 500".to_string()));
    assert_eq!(store_tx.option_type, Some("CALL".to_string()));
    assert_eq!(store_tx.strike, Some(5200.0));
    assert!(store_tx.expiry.is_some());
}

#[test]
fn test_transaction_list_from_account_transactions() {
    // Create a vector of mock AccountTransactions
    let account_txs = vec![
        AccountTransaction {
            date: "2025-05-15T10:30:00".to_string(),
            date_utc: "2025-05-15T10:30:00".to_string(),
            open_date_utc: "".to_string(),
            instrument_name: "GOLD CALL 2000".to_string(),
            period: "MAY-25".to_string(),
            profit_and_loss: "E100.50".to_string(),
            transaction_type: "DEAL".to_string(),
            reference: "ABCD1234".to_string(),
            open_level: "1950.5".to_string(),
            close_level: "2050.75".to_string(),
            size: "1".to_string(),
            currency: "EUR".to_string(),
            cash_transaction: false,
        },
        AccountTransaction {
            date: "2025-05-16T11:45:00".to_string(),
            date_utc: "2025-05-16T11:45:00".to_string(),
            open_date_utc: "".to_string(),
            instrument_name: "US500 PUT 4500".to_string(),
            period: "15-MAY-25".to_string(),
            profit_and_loss: "E-75.25".to_string(),
            transaction_type: "DEAL".to_string(),
            reference: "EFGH5678".to_string(),
            open_level: "4600".to_string(),
            close_level: "4450".to_string(),
            size: "1".to_string(),
            currency: "EUR".to_string(),
            cash_transaction: false,
        },
    ];

    // Convert to TransactionList
    let tx_list = TransactionList::from(&account_txs);

    // Verify the conversion
    assert_eq!(tx_list.as_ref().len(), 2);

    // Now we can directly access the fields
    assert_eq!(tx_list.as_ref()[0].reference, "ABCD1234");
    assert_eq!(tx_list.as_ref()[0].pnl_eur, 100.50);
    assert_eq!(tx_list.as_ref()[1].reference, "EFGH5678");
    assert_eq!(tx_list.as_ref()[1].pnl_eur, -75.25);
}

#[test]
fn test_profit_and_loss_parsing_with_comma_separators() {
    // Test case 1: Value with comma separator (thousands)
    let account_tx_with_comma = AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "2025-05-15T09:00:00".to_string(),
        instrument_name: "Daily Gold (Aug Future) 2000 CALL".to_string(),
        period: "MAY-25".to_string(),
        profit_and_loss: "E1,000.50".to_string(), // Value with comma
        transaction_type: "DEAL".to_string(),
        reference: "TEST1234".to_string(),
        open_level: "1950.5".to_string(),
        close_level: "2050.75".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    // Test case 2: Value without comma separator
    let account_tx_without_comma = AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "2025-05-15T09:00:00".to_string(),
        instrument_name: "Daily Gold (Aug Future) 2000 CALL".to_string(),
        period: "MAY-25".to_string(),
        profit_and_loss: "E300.00".to_string(), // Value without comma
        transaction_type: "DEAL".to_string(),
        reference: "TEST5678".to_string(),
        open_level: "1950.5".to_string(),
        close_level: "2050.75".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    // Test case 3: Large value with multiple comma separators
    let account_tx_large_value = AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "2025-05-15T09:00:00".to_string(),
        instrument_name: "Daily Gold (Aug Future) 2000 CALL".to_string(),
        period: "MAY-25".to_string(),
        profit_and_loss: "E1,234,567.89".to_string(), // Large value with multiple commas
        transaction_type: "DEAL".to_string(),
        reference: "TEST9999".to_string(),
        open_level: "1950.5".to_string(),
        close_level: "2050.75".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    // Convert to StoreTransaction and verify parsing
    let store_tx1 = StoreTransaction::from(account_tx_with_comma);
    let store_tx2 = StoreTransaction::from(account_tx_without_comma);
    let store_tx3 = StoreTransaction::from(account_tx_large_value);

    // Verify that comma-separated values are parsed correctly
    assert_eq!(store_tx1.pnl_eur, 1000.50, "Failed to parse E1,000.50");
    assert_eq!(store_tx2.pnl_eur, 300.00, "Failed to parse E300.00");
    assert_eq!(
        store_tx3.pnl_eur, 1234567.89,
        "Failed to parse E1,234,567.89"
    );
}

#[test]
fn test_transaction_list_as_ref() {
    // Create a simple TransactionList with mock data
    let store_tx1 = StoreTransaction::from(AccountTransaction {
        date: "2025-05-15T10:30:00".to_string(),
        date_utc: "2025-05-15T10:30:00".to_string(),
        open_date_utc: "".to_string(),
        instrument_name: "GOLD CALL 2000".to_string(),
        period: "MAY-25".to_string(),
        profit_and_loss: "E100.50".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "ABCD1234".to_string(),
        open_level: "1950.5".to_string(),
        close_level: "2050.75".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    });

    let store_tx2 = StoreTransaction::from(AccountTransaction {
        date: "2025-05-16T11:45:00".to_string(),
        date_utc: "2025-05-16T11:45:00".to_string(),
        open_date_utc: "".to_string(),
        instrument_name: "US500 PUT 4500".to_string(),
        period: "15-MAY-25".to_string(),
        profit_and_loss: "E-75.25".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "EFGH5678".to_string(),
        open_level: "4600".to_string(),
        close_level: "4450".to_string(),
        size: "1".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    });

    let tx_list = TransactionList(vec![store_tx1, store_tx2]);

    // Test the as_ref implementation
    let slice = tx_list.as_ref();
    assert_eq!(slice.len(), 2);

    // Now we can directly access the fields
    assert_eq!(slice[0].reference, "ABCD1234");
    assert_eq!(slice[1].reference, "EFGH5678");

    // We can also test serialization/deserialization
    let json = serde_json::to_string(&slice[0]).unwrap();
    let deserialized: StoreTransaction = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.reference, "ABCD1234");
}
