use csv::{ReaderBuilder, Trim};
use core::panic;
use std::io;

mod csv_types;
use crate::csv_types::*;

mod engine_types;
use crate::engine_types::*;

mod engine;
use crate::engine::*;

#[cfg(test)]
mod tests;

fn main() -> Result<(), io::Error> {
    // NOTE: This is pretty primitive. Anything more complicated than a single arg I'd swap to using clap.
    let args = std::env::args().collect::<Vec<String>>();
    let Some(path) = args.get(1) else {
        panic!("No args specified. Please pass a single argument for the input file path.");
    };

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
        writer.serialize(account)?;
    }

    Ok(())
}
