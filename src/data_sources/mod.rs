pub mod csv;

use crate::UserTransactions;

pub trait DataSource {
    fn read_actions<'a>(
        &'a mut self,
    ) -> Result<Box<dyn Iterator<Item = UserTransactions> + 'a>, Box<dyn std::error::Error>>;
}
