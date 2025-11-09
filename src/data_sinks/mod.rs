pub mod csv;

use crate::UserAccount;

pub trait DataSink {
    fn write_accounts(&mut self, accounts: Vec<&UserAccount>) -> Result<(), String>;
}
