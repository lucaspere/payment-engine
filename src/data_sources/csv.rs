use std::path::Path;

use crate::{UserTransactions, data_sources::DataSource};

pub struct CsvDataSource {
    path: String,
}

impl CsvDataSource {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl DataSource for CsvDataSource {
    fn read_transactions<'a>(
        &'a mut self,
    ) -> Result<Box<dyn Iterator<Item = UserTransactions> + 'a>, Box<dyn std::error::Error>> {
        let path = Path::new(&self.path);
        let rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_path(path)?;

        let iter = rdr
            .into_deserialize::<UserTransactions>()
            .filter_map(|result| match result {
                Ok(action) => Some(action),
                Err(e) => {
                    eprintln!("Error reading record: {}", e);
                    None
                }
            });

        Ok(Box::new(iter))
    }
}
