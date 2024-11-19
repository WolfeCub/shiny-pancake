use std::collections::HashSet;

use serde::Serialize;

use crate::{CsvRow, CsvTransaction};

#[derive(Debug)]
pub enum ValidatedTransaction {
    Deposit { tx: u32, amount: f32 },
    Withdrawal { tx: u32, amount: f32 },
    Dispute { tx: u32 },
    Resolve { tx: u32 },
    Chargeback { tx: u32 },
}

impl TryFrom<&CsvRow> for ValidatedTransaction {
    type Error = ();

    fn try_from(transaction: &CsvRow) -> Result<Self, Self::Error> {
        Ok(match (&transaction.transaction_type, transaction.amount) {
            (CsvTransaction::Deposit, Some(amount)) => ValidatedTransaction::Deposit {
                tx: transaction.tx,
                amount,
            },
            (CsvTransaction::Withdrawal, Some(amount)) => ValidatedTransaction::Withdrawal {
                tx: transaction.tx,
                amount,
            },
            (CsvTransaction::Dispute, None) => ValidatedTransaction::Dispute { tx: transaction.tx },
            (CsvTransaction::Resolve, None) => ValidatedTransaction::Resolve { tx: transaction.tx },
            (CsvTransaction::Chargeback, None) => {
                ValidatedTransaction::Chargeback { tx: transaction.tx }
            }
            // Only deposit and widthdrawls should have an amount. So if those are missing we'll
            // assume a mistake from the partner's side.
            (CsvTransaction::Deposit | CsvTransaction::Withdrawal, None) => return Err(()),
            // Everything else shouldn't have an amount. So if it does we'll also assume it was a
            // partner mistake.
            (
                CsvTransaction::Dispute | CsvTransaction::Resolve | CsvTransaction::Chargeback,
                Some(_),
            ) => return Err(()),
        })
    }
}

fn four_decimal_places<S: serde::Serializer>(
    value: &f32,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let rounded = (value * 10000.0).round() / 10000.0;
    serializer.serialize_f32(rounded)
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Account {
    pub client: u16,

    #[serde(serialize_with = "four_decimal_places")]
    pub available: f32,
    #[serde(serialize_with = "four_decimal_places")]
    pub held: f32,
    #[serde(serialize_with = "four_decimal_places")]
    pub total: f32,

    pub locked: bool,

    #[serde(skip_serializing)]
    pub disputed_transactions: HashSet<u32>,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,

            disputed_transactions: HashSet::new(),
        }
    }

    pub fn deposit(&mut self, amount: f32) {
        if !self.locked {
            self.available += amount;
        }
    }

    pub fn widthdraw(&mut self, amount: f32) {
        if !self.locked && self.available - amount >= 0.0 {
            self.available -= amount;
        }
    }

    pub fn dispute(&mut self, tx: u32, disputed_transaction: &ValidatedTransaction) {
        match disputed_transaction {
            ValidatedTransaction::Deposit { amount, .. } => {
                self.available -= amount;
                self.held += amount;

                self.disputed_transactions.insert(tx);
            }
            ValidatedTransaction::Withdrawal { amount, .. } => {
                self.held += amount;

                self.disputed_transactions.insert(tx);
            }
            _ => {}
        }
    }

    pub fn resolve(&mut self, tx: u32, disputed_transaction: &ValidatedTransaction) {
        if !self.disputed_transactions.contains(&tx) {
            return;
        }

        match disputed_transaction {
            ValidatedTransaction::Deposit { amount, .. } => {
                self.available += amount;
                self.held -= amount;

                self.disputed_transactions.remove(&tx);
            }
            ValidatedTransaction::Withdrawal { amount, .. } => {
                self.held -= amount;

                self.disputed_transactions.remove(&tx);
            }
            _ => {}
        }
    }

    pub fn chargeback(&mut self, tx: u32, disputed_transaction: &ValidatedTransaction) {
        if !self.disputed_transactions.contains(&tx) {
            return;
        }

        match disputed_transaction {
            ValidatedTransaction::Deposit { amount, .. } => {
                self.held -= amount;
                self.locked = true;
            }
            ValidatedTransaction::Withdrawal { amount, .. } => {
                self.held -= amount;
                self.available += amount;

                self.locked = true;
            }
            _ => {}
        }
    }
}
