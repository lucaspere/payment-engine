use std::process;

use payment_engine::{
    PaymentEngine,
    data_sinks::{DataSink, csv::CsvDataSink},
    data_sources::{DataSource, csv::CsvDataSource},
};

fn main() {
    let file = std::env::args()
        .nth(1)
        .expect("Input file path required as first argument");
    let output = std::env::args().nth(2);

    let mut data_source = Box::new(CsvDataSource::new(file));

    let mut engine = PaymentEngine::new();

    match data_source.read_transactions() {
        Ok(actions) => {
            for action in actions {
                engine.process_action(action);
            }
        }
        Err(e) => {
            eprintln!("Failed to read data: {}", e);
            process::exit(1);
        }
    }

    let accounts: Vec<_> = engine.accounts.values().collect();

    let mut data_sink: Box<dyn DataSink> = match output {
        Some(path) => {
            let file = std::fs::File::create(&path).unwrap_or_else(|e| {
                eprintln!("Failed to create output file '{}': {}", path, e);
                process::exit(1);
            });
            Box::new(CsvDataSink::new(file))
        }
        None => Box::new(CsvDataSink::new(std::io::stdout())),
    };

    if let Err(e) = data_sink.write_accounts(accounts) {
        eprintln!("Failed to write output: {}", e);
        process::exit(1);
    }
}
