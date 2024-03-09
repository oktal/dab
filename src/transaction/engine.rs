use std::collections::HashMap;

use super::{Account, ClientId, Transaction, TransactionId, TransactionOperation};

#[derive(Debug)]
struct TransactionEntry {
    /// Amount of the transaction
    amount: f64,

    /// Flag indicated whether a transaction has been disputed or not
    disputed: bool,
}

#[derive(Debug)]
struct ClientEntry {
    /// Client that this entry refers to
    id: ClientId,

    /// The total funds that are available for trading, staking, withdrawal, etc
    available: f64,

    /// The total funds that are held for dispute
    held: f64,

    /// The total funds that are available or held
    total: f64,

    /// Whether the account is locked. An account is locked if a charge back occurs
    locked: bool,

    /// Transactions that have been processed
    transactions: HashMap<TransactionId, TransactionEntry>,
}

impl Into<Account> for ClientEntry {
    fn into(self) -> Account {
        Account {
            client: self.id,
            available: self.available,
            held: self.held,
            total: self.total,
            locked: self.locked,
        }
    }
}

impl ClientEntry {
    fn new(id: ClientId) -> Self {
        Self {
            id,
            available: Default::default(),
            held: Default::default(),
            total: Default::default(),
            locked: Default::default(),
            transactions: Default::default(),
        }
    }

    fn apply(&mut self, transaction: Transaction) {
        let id = transaction.id;

        match transaction.operation {
            TransactionOperation::Deposit(amount) => {
                if let Some(this) = self.acquire_mut(id) {
                    this.available += amount;
                    this.total += amount;
                }
            }

            TransactionOperation::Withdrawal(amount) => {
                if let Some(this) = self.acquire_mut(id) {
                    let available = this.available - amount;
                    if available >= 0.0 {
                        this.available = available;
                        this.total -= amount;
                    }
                }
            }

            TransactionOperation::Dispute => {
                if let Some(disputed_tx) = self.transactions.get_mut(&id) {
                    if !disputed_tx.disputed {
                        self.available -= disputed_tx.amount;
                        self.held += disputed_tx.amount;
                        disputed_tx.disputed = true;
                    }
                }
            }

            TransactionOperation::Resolve => {
                if let Some(disputed_tx) = self.transactions.get_mut(&id) {
                    if disputed_tx.disputed {
                        self.available += disputed_tx.amount;
                        self.held -= disputed_tx.amount;
                        disputed_tx.disputed = false;
                    }
                }
            }

            TransactionOperation::Chargeback => {
                if let Some(disputed_tx) = self.transactions.get(&id) {
                    if disputed_tx.disputed {
                        self.held -= disputed_tx.amount;
                        self.total -= disputed_tx.amount;

                        self.locked = true;
                    }
                }
            }
        }

        self.ensure_invariants();
    }

    fn as_account(&self) -> Account {
        Account {
            client: self.id,
            available: self.available,
            held: self.held,
            total: self.total,
            locked: self.locked,
        }
    }

    fn ensure_invariants(&self) {
        debug_assert!(
            self.available >= 0.0,
            "Available founds should always be >="
        );

        debug_assert!(self.total >= 0.0, "Total founds should always be >= 0");

        debug_assert!(
            self.total >= self.available,
            "Total founds should always be >= available founds"
        );
    }

    fn acquire_mut(&mut self, id: TransactionId) -> Option<&mut Self> {
        if self.transactions.contains_key(&id) {
            return None;
        }

        return Some(self);
    }
}

/// Main transaction engine that will process transactions
pub struct TransactionEngine {
    clients: HashMap<ClientId, ClientEntry>,
}

impl TransactionEngine {
    /// Create a new, empty transaction engine
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Process a transaction
    pub fn process(&mut self, transaction: Transaction) {
        let entry = self
            .clients
            .entry(transaction.client)
            .or_insert_with_key(|id| ClientEntry::new(*id));
        entry.apply(transaction);
    }

    /// Retrieve an iterator over all the current [`Account`] accounts
    pub fn accounts<'a>(&'a self) -> impl Iterator<Item = Account> + 'a {
        self.clients.values().map(ClientEntry::as_account)
    }
}
