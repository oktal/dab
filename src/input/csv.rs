use std::{fs::File, path::Path};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::transaction::{Transaction, TransactionOperation};

use super::Reader;

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct CsvTransactionRecord {
    r#type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

impl TryInto<Transaction> for CsvTransactionRecord {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Transaction, Self::Error> {
        let operation = match self.r#type {
            TransactionType::Deposit => TransactionOperation::Deposit(
                self.amount
                    .ok_or(anyhow!("deposit transaction should have an amount"))?,
            ),

            TransactionType::Withdrawal => TransactionOperation::Withdrawal(
                self.amount
                    .ok_or(anyhow!("withdrawal transaction should have an amount"))?,
            ),

            TransactionType::Dispute => TransactionOperation::Dispute,
            TransactionType::Resolve => TransactionOperation::Resolve,
            TransactionType::Chargeback => TransactionOperation::Chargeback,
        };

        Ok(Transaction {
            client: self.client.into(),
            id: self.tx.into(),
            operation,
        })
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
    type IntoError = anyhow::Error;
    type Item = CsvTransactionRecord;
    type Error = csv::Error;

    type Iterator = csv::DeserializeRecordsIntoIter<File, CsvTransactionRecord>;

    fn into_iter(self) -> Self::Iterator {
        self.it
    }
}
