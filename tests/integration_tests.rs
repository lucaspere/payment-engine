use payment_engine::{
    PaymentEngine,
    data_sources::{DataSource, csv::CsvDataSource},
};
use rust_decimal_macros::dec;

#[test]
fn test_transactions_csv() {
    let mut data_source = Box::new(CsvDataSource::new("test_transactions.csv".to_string()));
    let mut engine = PaymentEngine::new();

    match data_source.read_transactions() {
        Ok(actions) => {
            for action in actions {
                engine.process_action(action);
            }
        }
        Err(e) => panic!("Failed to read data: {}", e),
    }

    // Client 1: deposit 1.0, deposit 2.0, withdrawal 1.5 = 1.5 available
    let account1 = engine.accounts.get(&1).unwrap();
    assert_eq!(account1.available, dec!(1.5));
    assert_eq!(account1.held, dec!(0.0));
    assert_eq!(account1.total, dec!(1.5));
    assert!(!account1.locked);

    // Client 2: deposit 2.0, withdrawal 3.0 (insufficient) = 2.0 available
    let account2 = engine.accounts.get(&2).unwrap();
    assert_eq!(account2.available, dec!(2.0));
    assert_eq!(account2.held, dec!(0.0));
    assert_eq!(account2.total, dec!(2.0));
    assert!(!account2.locked);
}

#[test]
fn test_insufficient_funds_csv() {
    let mut data_source = Box::new(CsvDataSource::new(
        "test_insufficient_funds.csv".to_string(),
    ));
    let mut engine = PaymentEngine::new();

    match data_source.read_transactions() {
        Ok(actions) => {
            for action in actions {
                engine.process_action(action);
            }
        }
        Err(e) => panic!("Failed to read data: {}", e),
    }

    // Client 1: deposit 10.0, withdrawal 5.0, withdrawal 10.0 (insufficient) = 5.0 available
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(5.0));
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.total, dec!(5.0));
    assert!(!account.locked);
}

#[test]
fn test_dispute_csv() {
    let mut data_source = Box::new(CsvDataSource::new("test_dispute.csv".to_string()));
    let mut engine = PaymentEngine::new();

    match data_source.read_transactions() {
        Ok(actions) => {
            for action in actions {
                engine.process_action(action);
            }
        }
        Err(e) => panic!("Failed to read data: {}", e),
    }

    // Client 1:
    // - deposit 10.0, dispute, resolve = 10.0 available
    // - deposit 5.0, dispute, chargeback = 5.0 held then removed
    // Final: 10.0 available, 0.0 held, but chargeback reduces by 5.0
    let account = engine.accounts.get(&1).unwrap();
    assert_eq!(account.available, dec!(5.0)); // 10.0 - 5.0 from chargeback
    assert_eq!(account.held, dec!(0.0));
    assert_eq!(account.total, dec!(5.0));
    assert!(account.locked);
}

#[test]
fn test_comprehensive_csv() {
    let mut data_source = Box::new(CsvDataSource::new("test_comprehensive.csv".to_string()));
    let mut engine = PaymentEngine::new();

    match data_source.read_transactions() {
        Ok(actions) => {
            for action in actions {
                engine.process_action(action);
            }
        }
        Err(e) => panic!("Failed to read data: {}", e),
    }

    // Client 1: deposit 10.0, withdrawal 2.5, dispute tx1, resolve tx1, deposit 20.0
    // = 10.0 - 2.5 + 20.0 = 27.5
    let account1 = engine.accounts.get(&1).unwrap();
    assert_eq!(account1.available, dec!(27.5));
    assert_eq!(account1.held, dec!(0.0));
    assert_eq!(account1.total, dec!(27.5));
    assert!(!account1.locked);

    // Client 2: deposit 5.0, dispute, chargeback
    // = 5.0 held, then chargeback removes from both held and available
    let account2 = engine.accounts.get(&2).unwrap();
    assert_eq!(account2.available, dec!(-5.0));
    assert_eq!(account2.held, dec!(0.0));
    assert_eq!(account2.total, dec!(-5.0));
    assert!(account2.locked);

    // Client 3: deposit 100.0, withdrawal 50.0, dispute tx4
    // = 100.0 - 50.0 = 50.0, then dispute moves 100.0 to held
    // available = 50.0 - 100.0 = -50.0, held = 100.0
    let account3 = engine.accounts.get(&3).unwrap();
    assert_eq!(account3.available, dec!(-50.0));
    assert_eq!(account3.held, dec!(100.0));
    assert_eq!(account3.total, dec!(50.0));
    assert!(!account3.locked);
}
