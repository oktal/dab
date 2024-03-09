use std::io;

use crate::transaction::Account;

use super::Writer;

pub struct CsvWriter<W>
where
    W: io::Write,
{
    writer: csv::Writer<W>,
}

impl<W> CsvWriter<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> anyhow::Result<Self>
    where
        W: io::Write,
    {
        let writer = csv::WriterBuilder::new()
            .delimiter(b',')
            .has_headers(true)
            .from_writer(writer);

        Ok(Self { writer })
    }
}

impl<W> Writer for CsvWriter<W>
where
    W: io::Write,
{
    type Error = csv::Error;

    fn write(&mut self, account: Account) -> Result<(), Self::Error> {
        self.writer.serialize(account)
    }
}
