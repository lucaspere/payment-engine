use payment_engine::{
    PaymentEngine,
    data_sources::{DataSource, csv::CsvDataSource},
};

fn main() {
    let file = std::env::args().nth(1).unwrap();

    let mut data_source = Box::new(CsvDataSource::new(file));

    let mut engine = PaymentEngine::new();

    match data_source.read_actions() {
        Ok(actions) => {
            for action in actions {
                engine.process_action(action);
            }
        }
        Err(e) => {
            eprintln!("Failed to read data: {}", e);
            std::process::exit(1);
        }
    }

    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    for account in engine.accounts.values() {
        wtr.serialize(account).expect("Failed to write account");
    }
    wtr.flush().expect("Failed to flush writer");
}
