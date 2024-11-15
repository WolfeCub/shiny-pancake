use std::
    collections::HashMap
;

use crate::{engine_types::*, Account, CsvRow};

pub struct Engine {
    lookup: HashMap<u16, Vec<ValidatedTransaction>>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            lookup: HashMap::new(),
        }
    }

    pub fn process(&mut self, transaction: CsvRow) {
        if let Ok(validated) = ValidatedTransaction::try_from(&transaction) {
            let entry = self
                .lookup
                .entry(transaction.client)
                .or_insert_with(|| vec![]);
            entry.push(validated);
        }
    }

    pub fn output(&self) -> Vec<Account> {
        let mut result = vec![];
        for (client, transactions) in self.lookup.iter() {
            if let Some(account) = Engine::colapse_user(*client, transactions) {
                result.push(account);
            }
        }
        result
    }

    fn colapse_user(client: u16, transactions: &Vec<ValidatedTransaction>) -> Option<Account> {
        // NOTE: We aren't copying the whole transaction here. We just take a reference to it.
        let transaction_by_id = transactions
            .iter()
            .filter_map(|t| match t {
                ValidatedTransaction::Deposit { tx, .. }
                | ValidatedTransaction::Withdrawal { tx, .. } => Some((*tx, t)),
                _ => None,
            })
            .collect::<HashMap<u32, &ValidatedTransaction>>();

        let mut account = Account::new(client);

        for t in transactions.iter() {
            match t {
                ValidatedTransaction::Deposit { amount, .. } => {
                    account.deposit(*amount);
                }
                ValidatedTransaction::Withdrawal { amount, .. } => {
                    account.widthdraw(*amount);
                }
                ValidatedTransaction::Dispute { tx, .. } => {
                    account.dispute(*tx, &transaction_by_id);
                }
                ValidatedTransaction::Resolve { tx, .. } => {
                    account.resolve(*tx, &transaction_by_id);
                }
                ValidatedTransaction::Chargeback { tx, .. } => {
                    account.chargeback(*tx, &transaction_by_id);
                }
            }
        }
        account.total = account.available + account.held;

        Some(account)
    }
}
