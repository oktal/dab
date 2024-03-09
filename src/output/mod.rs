use crate::transaction::Account;
pub mod csv;
pub use csv::CsvWriter;

/// An abstraction to display or write accounts
pub trait Writer {
    /// Error type returned by the writer
    type Error: Into<Box<dyn std::error::Error>>;

    fn write(&mut self, account: Account) -> Result<(), Self::Error>;
}
