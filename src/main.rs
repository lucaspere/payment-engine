use rust_decimal::{Decimal, prelude::Zero};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum TxAction {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    ChargeBack,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UserActions {
    #[serde(rename = "type")]
    tx_action: TxAction,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    tx_id: u32,
    amount: Option<Decimal>,
}

fn serialize_to_four_places<S>(t: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let four_place_decimal = t.round_sf(4);
    serializer.serialize_some(&four_place_decimal)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct UserAccount {
    client_id: u16,
    #[serde(serialize_with = "serialize_to_four_places")]
    available: Decimal,
    #[serde(serialize_with = "serialize_to_four_places")]
    held: Decimal,
    #[serde(serialize_with = "serialize_to_four_places")]
    total: Decimal,
    locked: bool,
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

struct PaymentEngine {
    accounts: HashMap<u16, UserAccount>,
    actions: HashMap<u16, HashMap<u32, Vec<UserActions>>>,
}

impl PaymentEngine {
    pub fn new() -> Self {
        Self {
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
            TxAction::Withdrawal => {
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
                    let action = match self
                        .actions
                        .get(&action.client_id)
                        .and_then(|acts| acts.get(&action.tx_id))
                    {
                        Some(act) => act,
                        None => return,
                    };

                    let amount = action
                        .last()
                        .and_then(|action| action.amount)
                        .unwrap_or(Decimal::zero());
                    account.available -= amount;
                    account.held += amount;
                    account.calculate_total();
                }
            }
            TxAction::Resolve => {
                if let Some(account) = self.accounts.get_mut(&action.client_id) {
                    let actions = match self
                        .actions
                        .get(&action.client_id)
                        .and_then(|acts| acts.get(&action.tx_id))
                    {
                        Some(act) => act,
                        None => return,
                    };
                    let disputed_action = actions
                        .iter()
                        .find(|action| action.tx_action == TxAction::Dispute);
                    if disputed_action.is_some() {
                        let deposit_action = actions
                            .iter()
                            .find(|action| action.tx_action == TxAction::Deposit);
                        if let Some(deposit_action) = deposit_action {
                            let amount = deposit_action.amount.unwrap_or(Decimal::zero());
                            account.held -= amount;
                            account.available += amount;
                            account.calculate_total();
                        }
                    }
                }
            }
            TxAction::ChargeBack => {
                if let Some(account) = self.accounts.get_mut(&action.client_id) {
                    let actions = match self
                        .actions
                        .get(&action.client_id)
                        .and_then(|acts| acts.get(&action.tx_id))
                    {
                        Some(act) => act,
                        None => return,
                    };
                    let disputed_action = actions
                        .iter()
                        .find(|action| action.tx_action == TxAction::Dispute);
                    if disputed_action.is_some() {
                        let deposit_action = actions
                            .iter()
                            .find(|action| action.tx_action == TxAction::Deposit);
                        if let Some(deposit_action) = deposit_action {
                            let amount = deposit_action.amount.unwrap_or(Decimal::zero());
                            account.held -= amount;
                            account.available -= amount;
                            account.calculate_total();
                            account.locked = true;
                        }
                    }
                }
            }
        }

        self.actions
            .entry(action.client_id)
            .or_insert_with(HashMap::new)
            .entry(action.tx_id)
            .or_insert_with(Vec::new)
            .push(action);
    }
}

fn main() {
    let file = std::env::args().nth(1).unwrap();
    dbg!(&file);
    let path = Path::new(&file);
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(path)
        .expect("Failed to open file");
    let mut engine = PaymentEngine::new();
    for result in rdr.deserialize::<UserActions>() {
        match result {
            Ok(action) => {
                engine.process_action(action);
            }
            Err(e) => eprintln!("Error reading record: {}", e),
        }
    }
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for account in engine.accounts.values() {
        wtr.serialize(account).expect("Failed to write account");
    }
    wtr.flush().expect("Failed to flush writer");
}
