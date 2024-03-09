use std::{fs::File, path::Path};

use serde::{Deserialize, Serialize};

use crate::transaction::{Transaction, TransactionType};

use super::Reader;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct CsvTransactionRecord {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

impl Into<Transaction> for CsvTransactionRecord {
    fn into(self) -> Transaction {
        Transaction {
            transaction_type: self.r#type,
            client: self.client.into(),
            id: self.tx.into(),
            amount: self.amount,
        }
    }
}

pub(super) struct CsvReader {
    it: csv::DeserializeRecordsIntoIter<File, CsvTransactionRecord>,
}

impl CsvReader {
    pub(super) fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b',')
            .trim(csv::Trim::All)
            .from_path(path)?;

        let it = reader.into_deserialize();

        Ok(Self { it })
    }
}
impl Reader for CsvReader {
    type Item = CsvTransactionRecord;
    type Error = csv::Error;

    type Iterator = csv::DeserializeRecordsIntoIter<File, CsvTransactionRecord>;

    fn into_iter(self) -> Self::Iterator {
        self.it
    }
}
