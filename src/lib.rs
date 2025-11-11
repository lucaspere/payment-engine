use rust_decimal::{Decimal, prelude::Zero};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod data_sinks;
pub mod data_sources;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserTransactions {
    #[serde(rename = "type")]
    pub tx_type: TxType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    pub amount: Option<Decimal>,
}

fn serialize_to_four_places<S>(t: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let formatted = format!("{:.4}", t);
    serializer.serialize_str(&formatted)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserAccount {
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(serialize_with = "serialize_to_four_places")]
    pub available: Decimal,
    #[serde(serialize_with = "serialize_to_four_places")]
    pub held: Decimal,
    #[serde(serialize_with = "serialize_to_four_places")]
    pub total: Decimal,
    pub locked: bool,
}

impl UserAccount {
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            available: Decimal::zero(),
            held: Decimal::zero(),
            total: Decimal::zero(),
            locked: false,
        }
    }

    pub fn calculate_total(&mut self) {
        self.total = self.available + self.held;
    }
}

pub struct PaymentEngine {
    pub accounts: HashMap<u16, UserAccount>,
    actions: HashMap<u16, HashMap<u32, Vec<UserTransactions>>>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            actions: HashMap::new(),
        }
    }

    fn get_or_create_account(&mut self, client_id: u16) -> &mut UserAccount {
        self.accounts
            .entry(client_id)
            .or_insert(UserAccount::new(client_id))
    }

    fn process_deposit(&mut self, action: &UserTransactions) {
        let account = self.get_or_create_account(action.client_id);
        account.available += action.amount.unwrap_or(Decimal::zero());
        account.calculate_total();
    }

    fn process_withdrawal(&mut self, action: &UserTransactions) {
        if let Some(account) = self.accounts.get_mut(&action.client_id) {
            let amount = action.amount.unwrap_or(Decimal::zero());
            if account.available >= amount {
                account.available -= amount;
                account.calculate_total();
            }
        }
    }

    fn process_dispute(&mut self, action: &UserTransactions) {
        let amount = match self
            .actions
            .get(&action.client_id)
            .and_then(|acts| acts.get(&action.tx_id))
        {
            Some(acts) => acts
                .iter()
                .find(|a| a.tx_type == TxType::Deposit || a.tx_type == TxType::Withdrawal)
                .and_then(|a| a.amount)
                .unwrap_or(Decimal::zero()),
            None => return,
        };

        let account = self.get_or_create_account(action.client_id);
        account.available -= amount;
        account.held += amount;
        account.calculate_total();
    }

    fn process_resolve(&mut self, action: &UserTransactions) {
        let amount = match self
            .actions
            .get(&action.client_id)
            .and_then(|acts| acts.get(&action.tx_id))
        {
            Some(acts) => {
                let has_dispute = acts.iter().any(|a| a.tx_type == TxType::Dispute);
                if !has_dispute {
                    return;
                }

                acts.iter()
                    .find(|a| a.tx_type == TxType::Deposit || a.tx_type == TxType::Withdrawal)
                    .and_then(|a| a.amount)
                    .unwrap_or(Decimal::zero())
            }
            None => return,
        };

        if let Some(account) = self.accounts.get_mut(&action.client_id) {
            account.held -= amount;
            account.available += amount;
            account.calculate_total();
        }
    }

    fn process_chargeback(&mut self, action: &UserTransactions) {
        let amount = match self
            .actions
            .get(&action.client_id)
            .and_then(|acts| acts.get(&action.tx_id))
        {
            Some(acts) => {
                let has_dispute = acts.iter().any(|a| a.tx_type == TxType::Dispute);
                if !has_dispute {
                    return;
                }

                acts.iter()
                    .find(|a| a.tx_type == TxType::Deposit || a.tx_type == TxType::Withdrawal)
                    .and_then(|a| a.amount)
                    .unwrap_or(Decimal::zero())
            }
            None => return,
        };
        if let Some(account) = self.accounts.get_mut(&action.client_id) {
            account.held -= amount;
            account.available -= amount;
            account.locked = true;
            account.calculate_total();
        }
    }
    pub fn process_action(&mut self, action: UserTransactions) {
        match action.tx_type {
            TxType::Deposit => self.process_deposit(&action),
            TxType::Withdrawal => self.process_withdrawal(&action),
            TxType::Dispute => self.process_dispute(&action),
            TxType::Resolve => self.process_resolve(&action),
            TxType::Chargeback => self.process_chargeback(&action),
        }

        self.actions
            .entry(action.client_id)
            .or_insert_with(HashMap::new)
            .entry(action.tx_id)
            .or_insert_with(Vec::new)
            .push(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_deposit_creates_account() {
        let mut engine = PaymentEngine::new();
        let action = UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        };
        engine.process_action(action);

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(100.0));
        assert_eq!(account.total, dec!(100.0));
    }

    #[test]
    fn test_multiple_deposits() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(50.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 2,
            amount: Some(dec!(75.5)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(125.5));
        assert_eq!(account.total, dec!(125.5));
    }

    #[test]
    fn test_withdrawal_with_sufficient_funds() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Withdrawal,
            client_id: 1,
            tx_id: 2,
            amount: Some(dec!(30.0)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(70.0));
        assert_eq!(account.total, dec!(70.0));
    }

    #[test]
    fn test_withdrawal_with_insufficient_funds() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(50.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Withdrawal,
            client_id: 1,
            tx_id: 2,
            amount: Some(dec!(100.0)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(50.0));
        assert_eq!(account.total, dec!(50.0));
    }

    #[test]
    fn test_withdrawal_nonexistent_account() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Withdrawal,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(50.0)),
        });

        assert!(engine.accounts.get(&1).is_none());
    }

    #[test]
    fn test_dispute_moves_funds_to_held() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(0.0));
        assert_eq!(account.held, dec!(100.0));
        assert_eq!(account.total, dec!(100.0));
    }

    #[test]
    fn test_resolve_returns_funds_to_available() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Resolve,
            client_id: 1,
            tx_id: 1,
            amount: None,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(100.0));
        assert_eq!(account.held, dec!(0.0));
        assert_eq!(account.total, dec!(100.0));
        assert!(!account.locked);
    }

    #[test]
    fn test_chargeback_locks_account() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Dispute,
            client_id: 1,
            tx_id: 1,
            amount: None,
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Chargeback,
            client_id: 1,
            tx_id: 1,
            amount: None,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.held, dec!(0.0));
        assert_eq!(account.total, dec!(-100.0));
        assert!(account.locked);
    }

    #[test]
    fn test_resolve_without_dispute_does_nothing() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Resolve,
            client_id: 1,
            tx_id: 1,
            amount: None,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(100.0));
        assert_eq!(account.held, dec!(0.0));
    }

    #[test]
    fn test_multiple_clients() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 2,
            tx_id: 2,
            amount: Some(dec!(200.0)),
        });

        assert_eq!(engine.accounts.get(&1).unwrap().total, dec!(100.0));
        assert_eq!(engine.accounts.get(&2).unwrap().total, dec!(200.0));
    }

    #[test]
    fn test_deposit_with_zero_amount() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(0.0)),
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(0.0));
    }

    #[test]
    fn test_dispute_nonexistent_transaction() {
        let mut engine = PaymentEngine::new();
        engine.process_action(UserTransactions {
            tx_type: TxType::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(dec!(100.0)),
        });
        engine.process_action(UserTransactions {
            tx_type: TxType::Dispute,
            client_id: 1,
            tx_id: 999,
            amount: None,
        });

        let account = engine.accounts.get(&1).unwrap();
        assert_eq!(account.available, dec!(100.0));
        assert_eq!(account.held, dec!(0.0));
    }
}
