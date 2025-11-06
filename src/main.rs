use std::collections::HashMap;

use rust_decimal::{Decimal, prelude::Zero};
enum TxAction {
    Deposit,
    Withdraw,
    Dispute,
}
struct UserActions {
    tx_action: TxAction,
    client_id: u16,
    tx_id: u32,
    amount: Option<rust_decimal::Decimal>,
}

#[derive(Debug)]
struct UserAccount {
    client_id: u16,
    available: rust_decimal::Decimal,
    held: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    locked: bool,
}

impl UserAccount {
    pub fn new(client_id: u16) -> Self {
        UserAccount {
            client_id,
            available: rust_decimal::Decimal::new(0, 0),
            held: rust_decimal::Decimal::new(0, 0),
            total: rust_decimal::Decimal::new(0, 0),
            locked: false,
        }
    }
    pub fn calculate_available(&mut self) {
        self.available = self.total - self.held;
    }
    pub fn calculate_total(&mut self) {
        self.total = self.available + self.held;
    }
    pub fn calculate_held(&mut self) {
        self.held = self.total - self.available;
    }
}

struct PaymentEngine {
    accounts: HashMap<u16, UserAccount>,
    actions: HashMap<u32, UserActions>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        PaymentEngine {
            accounts: HashMap::new(),
            actions: HashMap::new(),
        }
    }

    pub fn process_action(&mut self, action: UserActions) {
        match action.tx_action {
            TxAction::Deposit => {
                let account = self
                    .accounts
                    .entry(action.client_id)
                    .or_insert(UserAccount::new(action.client_id));
                account.available += action.amount.unwrap_or(Decimal::zero());
                account.calculate_total();
            }
            TxAction::Withdraw => {
                if let Some(account) = self.accounts.get_mut(&action.client_id) {
                    let amount = action.amount.unwrap_or(Decimal::zero());
                    if account.available >= amount {
                        account.available -= amount;
                        account.calculate_total();
                    }
                }
            }
            TxAction::Dispute => {
                if let Some(account) = self.accounts.get_mut(&action.client_id) {
                    let action = match self.actions.get(&action.tx_id) {
                        Some(act) => act,
                        None => return,
                    };

                    let amount = action.amount.unwrap_or(Decimal::zero());
                    account.available -= amount;
                    account.held += amount;
                    account.calculate_total();
                }
            }
            _ => {}
        }

        self.actions.insert(action.tx_id, action);
    }
}

fn main() {
    let mock_data = vec![
        UserActions {
            tx_action: TxAction::Deposit,
            client_id: 1,
            tx_id: 1,
            amount: Some(Decimal::new(1000, 2)),
        },
        UserActions {
            tx_action: TxAction::Withdraw,
            client_id: 1,
            tx_id: 2,
            amount: Some(Decimal::new(500, 2)),
        },
        UserActions {
            tx_action: TxAction::Deposit,
            client_id: 2,
            tx_id: 3,
            amount: Some(Decimal::new(2000, 2)),
        },
        UserActions {
            tx_action: TxAction::Dispute,
            client_id: 1,
            tx_id: 2,
            amount: None,
        },
    ];
    let mut engine = PaymentEngine::new();
    for action in mock_data {
        engine.process_action(action);
    }
    dbg!(&engine.accounts);
    println!("Hello, world!");
}
