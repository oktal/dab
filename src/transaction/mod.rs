use serde::{Deserialize, Serialize};

pub mod engine;

/// Represents a type of transaction handled by the payment engine
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransactionOperation {
    /// A deposit is a credit to the client's asset account
    Deposit(f64),

    /// A withdrawl is a debit to the client's asset account
    Withdrawal(f64),

    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed
    Dispute,

    /// A resolve represents a resolution to a dispute, releasing the associated held funds.
    /// Funds that were previously disputed are no longer disputed.
    Resolve,

    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn.
    Chargeback,
}

/// A unique identifier for a client that identifies a client's identity
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ClientId(u16);

impl From<u16> for ClientId {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

/// A unique identifier for a transaction
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransactionId(u32);

impl From<u32> for TransactionId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Represents a transaction that occured for a particular client
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Client identifier
    pub client: ClientId,

    /// Transaction identifier
    /// A transaction id can either be unique for deposit and withdrawal transactions
    /// or represent a reference to an other transaction for other transaction types
    pub id: TransactionId,

    /// The operation conveyed by the transaction
    pub operation: TransactionOperation,
}

/// Represents an account for a particular client
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Client that this account is associated with
    pub client: ClientId,

    /// The total funds that are available for trading, staking, withdrawal, etc
    pub available: f64,

    /// The total funds that are held for dispute
    pub held: f64,

    /// The total funds that are available or held
    pub total: f64,

    /// Whether the account is locked. An account is locked if a charge back occurs
    pub locked: bool,
}
