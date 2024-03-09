use std::path::Path;

use anyhow::anyhow;

use crate::transaction::Transaction;

mod csv;

/// An abstraction to read transaction records
pub trait Reader {
    /// Type that the reader will yield that must be convertible to a [`Transaction`]
    type Item: Into<Transaction>;

    /// Error raised by the reader
    type Error: Into<Box<dyn std::error::Error>>;

    /// An iterator type that can be used to iterator over the [`Self::Item`] elements from the reader
    /// The iterator will yield a [`Result`] over the elements
    type Iterator: Iterator<Item = Result<Self::Item, Self::Error>>;

    /// Convert the reader to an iterator over the [`Self::Item`] elements
    fn into_iter(self) -> Self::Iterator;
}

/// Read transactions from a CSV file
/// Returns a success iterator over the [`Transaction`] read from the CSV file or an IO error
pub fn read_csv(
    path: impl AsRef<Path>,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<Transaction>>> {
    Ok(read(csv::CsvReader::new(path)?))
}

/// Read transactions from a [`Reader`]
/// Returns an iterator over the [`Transaction`] read from the reader
fn read<R: Reader>(reader: R) -> impl Iterator<Item = anyhow::Result<Transaction>> {
    reader
        .into_iter()
        .map(|e| e.map(Into::into).map_err(|e| anyhow!("{}", e.into())))
}
