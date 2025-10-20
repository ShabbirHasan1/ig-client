use chrono::Utc;
use ig_client::presentation::account::AccountTransaction;
use ig_client::presentation::transaction::{StoreTransaction, TransactionList};

#[test]
fn test_store_transaction_default() {
    let transaction = StoreTransaction::default();

    assert_eq!(transaction.underlying, None);
    assert_eq!(transaction.strike, None);
    assert_eq!(transaction.option_type, None);
    assert_eq!(transaction.expiry, None);
    assert_eq!(transaction.transaction_type, "");
    assert_eq!(transaction.pnl_eur, 0.0);
    assert_eq!(transaction.reference, "");
    assert!(!transaction.is_fee);
    assert_eq!(transaction.raw_json, "");
}

#[test]
fn test_store_transaction_clone() {
    let transaction = StoreTransaction {
        deal_date: Utc::now(),
        underlying: Some("GOLD".to_string()),
        strike: Some(1800.0),
        option_type: Some("CALL".to_string()),
        expiry: None,
        transaction_type: "DEAL".to_string(),
        pnl_eur: 100.50,
        reference: "REF123".to_string(),
        is_fee: false,
        raw_json: "{}".to_string(),
    };

    let cloned = transaction.clone();
    assert_eq!(transaction.underlying, cloned.underlying);
    assert_eq!(transaction.strike, cloned.strike);
    assert_eq!(transaction.pnl_eur, cloned.pnl_eur);
    assert_eq!(transaction.reference, cloned.reference);
}

#[test]
fn test_store_transaction_serialization() {
    let transaction = StoreTransaction {
        deal_date: Utc::now(),
        underlying: Some("US500".to_string()),
        strike: None,
        option_type: None,
        expiry: None,
        transaction_type: "DEAL".to_string(),
        pnl_eur: -50.25,
        reference: "REF456".to_string(),
        is_fee: false,
        raw_json: r#"{"test":"data"}"#.to_string(),
    };

    let json = serde_json::to_string(&transaction).unwrap();
    let deserialized: StoreTransaction = serde_json::from_str(&json).unwrap();

    assert_eq!(transaction.underlying, deserialized.underlying);
    assert_eq!(transaction.pnl_eur, deserialized.pnl_eur);
    assert_eq!(transaction.reference, deserialized.reference);
}

#[test]
fn test_store_transaction_display() {
    let transaction = StoreTransaction {
        deal_date: Utc::now(),
        underlying: Some("GOLD".to_string()),
        strike: Some(1800.0),
        option_type: Some("PUT".to_string()),
        expiry: None,
        transaction_type: "DEAL".to_string(),
        pnl_eur: 200.75,
        reference: "REF789".to_string(),
        is_fee: false,
        raw_json: "{}".to_string(),
    };

    let display = format!("{}", transaction);
    assert!(!display.is_empty());
}

#[test]
fn test_store_transaction_partial_eq() {
    let tx1 = StoreTransaction {
        deal_date: Utc::now(),
        underlying: Some("GOLD".to_string()),
        strike: Some(1800.0),
        option_type: Some("CALL".to_string()),
        expiry: None,
        transaction_type: "DEAL".to_string(),
        pnl_eur: 100.0,
        reference: "REF123".to_string(),
        is_fee: false,
        raw_json: "{}".to_string(),
    };

    let tx2 = tx1.clone();
    assert_eq!(tx1, tx2);
}

#[test]
fn test_store_transaction_from_account_transaction() {
    let account_tx = AccountTransaction {
        date: "2024-01-15".to_string(),
        date_utc: "2024-01-15T10:30:00".to_string(),
        open_date_utc: "2024-01-15T09:00:00".to_string(),
        instrument_name: "GOLD".to_string(),
        period: "JAN-24".to_string(),
        profit_and_loss: "E100.50".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "REF123".to_string(),
        open_level: "1800.0".to_string(),
        close_level: "1850.0".to_string(),
        size: "1.0".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    let store_tx = StoreTransaction::from(account_tx);

    assert_eq!(store_tx.underlying, Some("GOLD".to_string()));
    assert_eq!(store_tx.transaction_type, "DEAL");
    assert_eq!(store_tx.pnl_eur, 100.50);
    assert_eq!(store_tx.reference, "REF123");
    assert!(!store_tx.is_fee);
}

#[test]
fn test_store_transaction_from_account_transaction_ref() {
    let account_tx = AccountTransaction {
        date: "2024-01-15".to_string(),
        date_utc: "2024-01-15T10:30:00".to_string(),
        open_date_utc: "2024-01-15T09:00:00".to_string(),
        instrument_name: "US500".to_string(),
        period: "-".to_string(),
        profit_and_loss: "E-50.25".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "REF456".to_string(),
        open_level: "".to_string(),
        close_level: "".to_string(),
        size: "".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    let store_tx = StoreTransaction::from(&account_tx);

    assert_eq!(store_tx.underlying, Some("US500".to_string()));
    assert_eq!(store_tx.pnl_eur, -50.25);
}

#[test]
fn test_store_transaction_is_fee_detection() {
    let account_tx = AccountTransaction {
        date: "2024-01-15".to_string(),
        date_utc: "2024-01-15T10:30:00".to_string(),
        open_date_utc: "".to_string(),
        instrument_name: "GOLD".to_string(),
        period: "-".to_string(),
        profit_and_loss: "E0.50".to_string(),
        transaction_type: "WITH".to_string(),
        reference: "FEE123".to_string(),
        open_level: "".to_string(),
        close_level: "".to_string(),
        size: "".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    let store_tx = StoreTransaction::from(account_tx);

    assert!(store_tx.is_fee);
    assert_eq!(store_tx.transaction_type, "WITH");
}

#[test]
fn test_store_transaction_pnl_parsing_with_comma() {
    let account_tx = AccountTransaction {
        date: "2024-01-15".to_string(),
        date_utc: "2024-01-15T10:30:00".to_string(),
        open_date_utc: "".to_string(),
        instrument_name: "GOLD".to_string(),
        period: "-".to_string(),
        profit_and_loss: "E1,234.56".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "REF789".to_string(),
        open_level: "".to_string(),
        close_level: "".to_string(),
        size: "".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    };

    let store_tx = StoreTransaction::from(account_tx);

    assert_eq!(store_tx.pnl_eur, 1234.56);
}

#[test]
fn test_transaction_list_from_vec() {
    let transactions = vec![
        AccountTransaction {
            date: "2024-01-15".to_string(),
            date_utc: "2024-01-15T10:30:00".to_string(),
            open_date_utc: "".to_string(),
            instrument_name: "GOLD".to_string(),
            period: "-".to_string(),
            profit_and_loss: "E100.00".to_string(),
            transaction_type: "DEAL".to_string(),
            reference: "REF1".to_string(),
            open_level: "".to_string(),
            close_level: "".to_string(),
            size: "".to_string(),
            currency: "EUR".to_string(),
            cash_transaction: false,
        },
        AccountTransaction {
            date: "2024-01-16".to_string(),
            date_utc: "2024-01-16T10:30:00".to_string(),
            open_date_utc: "".to_string(),
            instrument_name: "US500".to_string(),
            period: "-".to_string(),
            profit_and_loss: "E-50.00".to_string(),
            transaction_type: "DEAL".to_string(),
            reference: "REF2".to_string(),
            open_level: "".to_string(),
            close_level: "".to_string(),
            size: "".to_string(),
            currency: "EUR".to_string(),
            cash_transaction: false,
        },
    ];

    let tx_list = TransactionList::from(&transactions);

    assert_eq!(tx_list.0.len(), 2);
    assert_eq!(tx_list.0[0].reference, "REF1");
    assert_eq!(tx_list.0[1].reference, "REF2");
}

#[test]
fn test_transaction_list_as_ref() {
    let transactions = vec![AccountTransaction {
        date: "2024-01-15".to_string(),
        date_utc: "2024-01-15T10:30:00".to_string(),
        open_date_utc: "".to_string(),
        instrument_name: "GOLD".to_string(),
        period: "-".to_string(),
        profit_and_loss: "E100.00".to_string(),
        transaction_type: "DEAL".to_string(),
        reference: "REF1".to_string(),
        open_level: "".to_string(),
        close_level: "".to_string(),
        size: "".to_string(),
        currency: "EUR".to_string(),
        cash_transaction: false,
    }];

    let tx_list = TransactionList::from(&transactions);
    let slice: &[StoreTransaction] = tx_list.as_ref();

    assert_eq!(slice.len(), 1);
    assert_eq!(slice[0].reference, "REF1");
}
