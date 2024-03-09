use std::io;

use anyhow::bail;
use output::Writer;
use transaction::engine::TransactionEngine;

mod input;
mod output;
mod transaction;

fn main() -> anyhow::Result<()> {
    let Some(transactions_file) = std::env::args().skip(1).next() else {
        bail!("usage bail [transactions_file]");
    };

    let transactions = input::read_csv(transactions_file)?;

    let mut engine = TransactionEngine::new();

    for transaction in transactions {
        let transaction = transaction?;
        engine.process(transaction);
    }

    let mut writer = output::CsvWriter::new(io::stdout())?;

    for account in engine.accounts() {
        writer.write(account)?;
    }

    Ok(())
}
