use std::collections::HashMap;
enum TxAction {
    Deposit,
    Withdraw,
    Trade,
    Stake,
}
struct UserActions {
    tx_action: TxAction,
    client_id: u16,
    tx_id: u32,
    amount: rust_decimal::Decimal,
}

struct UserAccount {
    client_id: u16,
    available: rust_decimal::Decimal,
    held: rust_decimal::Decimal,
    total: rust_decimal::Decimal,
    locked: bool,
}

impl UserAccount {
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

fn main() {
    println!("Hello, world!");
}
