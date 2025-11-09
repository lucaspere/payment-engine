use crate::{UserTransactions, data_sources::DataSource};

pub struct MemoryDataSource {
    actions: Vec<UserTransactions>,
}

impl MemoryDataSource {
    pub fn new(actions: Vec<UserTransactions>) -> Self {
        Self { actions }
    }
}

impl DataSource for MemoryDataSource {
    fn read_actions<'a>(
        &'a mut self,
    ) -> Result<Box<dyn Iterator<Item = UserTransactions> + 'a>, Box<dyn std::error::Error>> {
        Ok(Box::new(self.actions.clone().into_iter()))
    }
}
