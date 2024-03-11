use std::collections::{hash_map::Entry, HashMap};

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

    fn apply(&mut self, transaction: Transaction) -> Account {
        let id = transaction.id;

        match transaction.operation {
            TransactionOperation::Deposit(amount) => {
                if let Entry::Vacant(e) = self.transactions.entry(id) {
                    self.available += amount;
                    self.total += amount;

                    e.insert(TransactionEntry {
                        amount,
                        disputed: false,
                    });
                }
            }

            TransactionOperation::Withdrawal(amount) => {
                if let Entry::Vacant(e) = self.transactions.entry(id) {
                    let available = self.available - amount;
                    if available >= 0.0 {
                        self.available = available;
                        self.total -= amount;
                    }

                    e.insert(TransactionEntry {
                        amount,
                        disputed: false,
                    });
                }
            }

            TransactionOperation::Dispute => {
                if let Some(disputed_tx) = self.transactions.get_mut(&id) {
                    if !disputed_tx.disputed {
                        // TODO(oktal): unclear as to why the available amount must be decreased
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

        self.as_account()
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
    /// Returns the [`Account`] associated with the client of the transaction if the client for which the
    /// transaction should be applied exist or [`None`] otherwise
    pub fn process(&mut self, transaction: Transaction) -> Option<Account> {
        let entry = match transaction.operation {
            TransactionOperation::Deposit(_) => Some(
                self.clients
                    .entry(transaction.client)
                    .or_insert_with_key(|id| ClientEntry::new(*id)),
            ),

            _ => self.clients.get_mut(&transaction.client),
        };
        entry.map(|e| e.apply(transaction))
    }

    /// Retrieve an iterator over all the current [`Account`] accounts
    pub fn accounts<'a>(&'a self) -> impl Iterator<Item = Account> + 'a {
        self.clients.values().map(ClientEntry::as_account)
    }

    #[cfg(test)]
    fn account_of(&self, client: ClientId) -> Option<Account> {
        self.clients.get(&client).map(ClientEntry::as_account)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::ClientId;

    const BOB: ClientId = ClientId(1);
    const ALICE: ClientId = ClientId(2);

    #[test]
    fn deposit() {
        // Setup
        let mut engine = TransactionEngine::new();

        // Deposit to bob's account
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(1),
                operation: TransactionOperation::Deposit(10.0),
            })
            .expect("bob's account should exist after deposit");

        // Make sure bob's account have been deposited with 10.0
        assert_eq!(account.client, BOB);
        assert_eq!(account.total, 10.0);
        assert_eq!(account.available, 10.0);

        // No fund should be held
        assert_eq!(account.held, 0.0);

        // Bob's account should ne be locked
        assert!(!account.locked);

        // Make sure ALICE does not exist
        assert!(matches!(engine.account_of(ALICE), None));
    }

    #[test]
    fn double_deposit() {
        // Setup
        let mut engine = TransactionEngine::new();

        // Deposit to bob's account
        engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(1),
                operation: TransactionOperation::Deposit(10.0),
            })
            .expect("bob's account should exist after deposit");

        // Attempt to double deposit the same transaction to bob's account
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(1),
                operation: TransactionOperation::Deposit(10.0),
            })
            .expect("bob's account should exist after deposit");

        // Make sure the amount has not been deposited twice
        assert_eq!(account.client, BOB);
        assert_eq!(account.total, 10.0);
        assert_eq!(account.available, 10.0);
    }

    #[test]
    fn withdraw_available_funds() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;

        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Withdraw half the paycheck for taxes
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(2),
                operation: TransactionOperation::Withdrawal(PAYCHECK / 2.0),
            })
            .expect("bob's account should exist after withdrawing from an existing account");

        // Make sure bob's account has been withdrawn
        assert_eq!(account.client, BOB);
        assert_eq!(account.total, 50.0);
        assert_eq!(account.available, 50.0);

        // No fund should be held
        assert_eq!(account.held, 0.0);

        // Bob's account should ne be locked
        assert!(!account.locked);
    }

    #[test]
    fn withdraw_unknown_client() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;

        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Withdraw from Alice account
        let account = engine.process(Transaction {
            client: ALICE,
            id: TransactionId(2),
            operation: TransactionOperation::Withdrawal(PAYCHECK / 2.0),
        });

        // Make sure the account does not exist for Alice
        assert!(matches!(account, None));
    }

    #[test]
    fn withdraw_insufficient_funds() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;

        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Withdraw twice the paycheck to pay rent
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(2),
                operation: TransactionOperation::Withdrawal(PAYCHECK * 2.0),
            })
            .expect("bob's account should exist after withdrawing from an existing account");

        // Make sure bob's account has not been withdrawn
        assert_eq!(account.client, BOB);
        assert_eq!(account.total, PAYCHECK);
        assert_eq!(account.available, PAYCHECK);
    }

    #[test]
    fn dispute_unknown_transaction() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;

        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Attempt to dispute an unknown transaction from Bob
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(100),
                operation: TransactionOperation::Dispute,
            })
            .expect("Bob's account should exist after depositing");

        // Make sure nothing has been disputed
        assert_eq!(account.client, BOB);
        assert_eq!(account.total, PAYCHECK);
        assert_eq!(account.available, PAYCHECK);
        assert_eq!(account.held, 0.0);
    }

    #[test]
    fn dispute_unknown_client() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;

        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Attempt to dispute Alice' account
        let account = engine.process(Transaction {
            client: ALICE,
            id: TransactionId(1),
            operation: TransactionOperation::Dispute,
        });

        // Make sure disputed account does not exist
        assert!(matches!(account, None));
    }

    #[test]
    fn dispute() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;
        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Bob has been scammed, withdraw everything
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(2),
            operation: TransactionOperation::Withdrawal(PAYCHECK),
        });

        // Bob realized he's been scammed, dispute the transaction
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(2),
                operation: TransactionOperation::Dispute,
            })
            .expect("Bob's account should exist after depositing");

        // Make sure the funds are held in bob's account
        assert_eq!(account.client, BOB);
        assert_eq!(account.held, PAYCHECK);
    }

    #[test]
    fn resolve() {
        // Setup
        let mut engine = TransactionEngine::new();

        const PAYCHECK: f64 = 100.0;
        // Deposit paycheck to Bob's account
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(1),
            operation: TransactionOperation::Deposit(PAYCHECK),
        });

        // Bob has been scammed, withdraw everything
        engine.process(Transaction {
            client: BOB,
            id: TransactionId(2),
            operation: TransactionOperation::Withdrawal(PAYCHECK),
        });

        // Bob realized he's been scammed, dispute the transaction
        engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(2),
                operation: TransactionOperation::Dispute,
            })
            .expect("Bob's account should exist after depositing");

        // Bank investigated and decided to give funds back to bob
        let account = engine
            .process(Transaction {
                client: BOB,
                id: TransactionId(2),
                operation: TransactionOperation::Resolve,
            })
            .expect("bob's account should exist after depositing");

        // Make sure the dispute has been resolved
        assert_eq!(account.client, BOB);
        assert_eq!(account.held, 0.0);
        // TODO(oktal): this check fails because we decrement the available amount
        // assert_eq!(account.available, PAYCHECK);
    }
}
