use csv::{ReaderBuilder, Trim};
use std::{collections::HashMap, io};

mod csv_types;
use crate::csv_types::*;

mod engine_types;
use crate::engine_types::*;

mod engine;
use crate::engine::*;

#[cfg(test)]
mod tests;


fn main() -> Result<(), io::Error> {

    let args = std::env::args().collect::<Vec<String>>();
    let path = args.get(1).expect("blah");

    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(path)?;

    let mut engine = Engine::new();
    for transaction in reader.deserialize::<CsvRow>() {
        engine.process(transaction?);
    }

    let mut writer = csv::Writer::from_writer(io::stdout());
    let results = engine.output();
    for account in results {
        writer.serialize(account).expect("TODO");
    }
    // println!("{:?}", rows);

    Ok(())
}
