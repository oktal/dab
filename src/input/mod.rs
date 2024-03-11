use std::path::Path;

use anyhow::anyhow;

use crate::transaction::Transaction;

mod csv;

/// An abstraction to read transaction records
pub trait Reader {
    /// Error raised when attempting to convert a record yielded by the reader to a [`Transaction`]
    type IntoError: Into<Box<dyn std::error::Error>>;

    /// Type that the reader will yield that must be convertible to a [`Transaction`]
    type Item: TryInto<Transaction, Error = Self::IntoError>;

    /// Error raised by the reader
    type Error: Into<Box<dyn std::error::Error>>;

    /// An iterator type that can be used to iterate over the [`Self::Item`] elements from the reader
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
    reader.into_iter().map(|record| match record {
        Ok(record) => record.try_into().map_err(|e| anyhow!("{}", e.into())),
        Err(e) => Err(anyhow!("{}", e.into())),
    })
}
