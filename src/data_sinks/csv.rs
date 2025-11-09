use std::io::Write;

use crate::{UserAccount, data_sinks::DataSink};

pub struct CsvDataSink<W: Write> {
    writer: csv::Writer<W>,
}

impl<W: Write> CsvDataSink<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: csv::Writer::from_writer(writer),
        }
    }
}

impl<W: Write> DataSink for CsvDataSink<W> {
    fn write_accounts(&mut self, accounts: Vec<&UserAccount>) -> Result<(), String> {
        for account in accounts {
            self.writer
                .serialize(account)
                .map_err(|e| format!("Failed to serialize account: {}", e))?;
        }
        self.writer
            .flush()
            .map_err(|e| format!("Failed to flush writer: {}", e))
    }
}
