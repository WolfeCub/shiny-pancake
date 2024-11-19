use std::collections::HashSet;

use crate::csv_types::*;
use crate::engine::*;
use crate::Account;


impl CsvRow {
    pub fn new(
        transaction_type: CsvTransaction,
        client: u16,
        tx: u32,
        amount: Option<f32>,
    ) -> Self {
        Self {
            transaction_type,
            client,
            tx,
            amount,
        }
    }
}

// Simple helper to reduce copy-paste and make our tests a bit more legible and look like CSV input
macro_rules! csv_input {
    ($($args:tt),* $(,)?) => {{
        let mut engine = Engine::new();

        $(engine.process(CsvRow::new$args);)*

        engine.output()
    }};
}

#[test]
fn simple_deposits() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(1.0)),
        (CsvTransaction::Deposit, 1, 2, Some(5.0)),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 6.0,
            held: 0.0,
            total: 6.0,
            locked: false,
            disputed_transactions: HashSet::new(),
        }]
    );
}

#[test]
fn multiple_deposits() {
    let mut output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(1.0)),
        (CsvTransaction::Deposit, 1, 2, Some(5.0)),
        (CsvTransaction::Deposit, 2, 3, Some(7.0)),
        (CsvTransaction::Deposit, 2, 4, Some(11.0)),
    );

    output.sort_by(|a, b| a.client.partial_cmp(&b.client).unwrap());

    assert_eq!(
        output,
        vec![
            Account {
                client: 1,
                available: 6.0,
                held: 0.0,
                total: 6.0,
                locked: false,
                disputed_transactions: HashSet::new(),
            },
            Account {
                client: 2,
                available: 18.0,
                held: 0.0,
                total: 18.0,
                locked: false,
                disputed_transactions: HashSet::new(),
            }
        ]
    );
}

#[test]
fn simple_widthdrawl() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(1.0)),
        (CsvTransaction::Deposit, 1, 2, Some(5.0)),
        (CsvTransaction::Withdrawal, 1, 2, Some(6.0)),
    );

    assert_eq!(output, vec![Account::new(1)]);
}

#[test]
fn simple_dispute() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(5.0)),
        (CsvTransaction::Deposit, 1, 2, Some(1.0)),
        (CsvTransaction::Dispute, 1, 1, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 1.0,
            held: 5.0,
            total: 6.0,
            locked: false,
            disputed_transactions: HashSet::from_iter([1]), // id 1 is never resolved
        }]
    );
}

#[test]
fn simple_dispute_resolve() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(5.0)),
        (CsvTransaction::Deposit, 1, 2, Some(1.0)),
        (CsvTransaction::Dispute, 1, 1, None),
        (CsvTransaction::Resolve, 1, 1, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 6.0,
            held: 0.0,
            total: 6.0,
            locked: false,
            disputed_transactions: HashSet::new(),
        }]
    );
}

#[test]
fn simple_dispute_chargeback() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(5.0)),
        (CsvTransaction::Deposit, 1, 2, Some(1.0)),
        (CsvTransaction::Dispute, 1, 1, None),
        (CsvTransaction::Chargeback, 1, 1, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 1.0,
            held: 0.0,
            total: 1.0,
            locked: true,
            disputed_transactions: HashSet::from_iter([1]), // id 1 is never resolved
        }]
    );
}

#[test]
fn resolve_no_dispute() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(5.0)),
        (CsvTransaction::Deposit, 1, 2, Some(1.0)),
        (CsvTransaction::Resolve, 1, 1, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 6.0,
            held: 0.0,
            total: 6.0,
            locked: false,
            disputed_transactions: HashSet::new(),
        }]
    );
}

#[test]
fn chargeback_no_dispute() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(5.0)),
        (CsvTransaction::Deposit, 1, 2, Some(1.0)),
        (CsvTransaction::Chargeback, 1, 1, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 6.0,
            held: 0.0,
            total: 6.0,
            locked: false,
            disputed_transactions: HashSet::new(),
        }]
    );
}

#[test]
fn dispute_double_resolved() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(5.0)),
        (CsvTransaction::Deposit, 1, 2, Some(1.0)),
        (CsvTransaction::Dispute, 1, 1, None),
        (CsvTransaction::Resolve, 1, 1, None),
        (CsvTransaction::Resolve, 1, 1, None), // We resolve it again but it shouldn't be under dispute anymore
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 6.0,
            held: 0.0,
            total: 6.0,
            locked: false,
            disputed_transactions: HashSet::new(),
        }]
    );
}

#[test]
fn dispute_withdrawal() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(10.0)),
        (CsvTransaction::Withdrawal, 1, 2, Some(3.0)),
        (CsvTransaction::Dispute, 1, 2, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 7.0,
            held: 3.0,
            total: 10.0,
            locked: false,
            disputed_transactions: HashSet::from_iter([2]),
        }]
    );
}

#[test]
fn dispute_resolve_withdrawal() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(10.0)),
        (CsvTransaction::Withdrawal, 1, 2, Some(3.0)),
        (CsvTransaction::Dispute, 1, 2, None),
        (CsvTransaction::Resolve, 1, 2, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 7.0,
            held: 0.0,
            total: 7.0,
            locked: false,
            disputed_transactions: HashSet::new(),
        }]
    );
}

#[test]
fn dispute_chargeback_withdrawal() {
    let output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(10.0)),
        (CsvTransaction::Withdrawal, 1, 2, Some(3.0)),
        (CsvTransaction::Dispute, 1, 2, None),
        (CsvTransaction::Chargeback, 1, 2, None),
    );

    assert_eq!(
        output,
        vec![Account {
            client: 1,
            available: 10.0,
            held: 0.0,
            total: 10.0,
            locked: true,
            disputed_transactions: HashSet::from_iter([2]),
        }]
    );
}

#[test]
fn multiple_all_operations() {
    let mut output = csv_input!(
        (CsvTransaction::Deposit, 1, 1, Some(1.7)),
        (CsvTransaction::Deposit, 1, 3, Some(7.0)),
        (CsvTransaction::Deposit, 2, 5, Some(4.1)),
        (CsvTransaction::Deposit, 2, 4, Some(3.7)),
        (CsvTransaction::Deposit, 4, 8, Some(9.3)),
        (CsvTransaction::Deposit, 3, 6, Some(6.9)),
        (CsvTransaction::Deposit, 3, 7, Some(5.6)),
        (CsvTransaction::Deposit, 1, 2, Some(2.5)),
        (CsvTransaction::Withdrawal, 4, 10, Some(3.4)),
        (CsvTransaction::Withdrawal, 2, 9, Some(1.5)),
        (CsvTransaction::Dispute, 3, 7, None),
        (CsvTransaction::Deposit, 3, 11, Some(3.0)),
        (CsvTransaction::Dispute, 1, 3, None),
        (CsvTransaction::Resolve, 3, 7, None),
        (CsvTransaction::Withdrawal, 3, 12, Some(2.0)),
        (CsvTransaction::Chargeback, 1, 3, None),
        (CsvTransaction::Dispute, 3, 6, None),
    );

    output.sort_by(|a, b| a.client.partial_cmp(&b.client).unwrap());

    let correct = vec![
        Account {
            client: 1,
            available: 4.2,
            held: 0.0,
            total: 4.2,
            locked: true,
            disputed_transactions: HashSet::from_iter([3]),
        },
        Account {
            client: 2,
            available: 6.3,
            held: 0.0,
            total: 6.3,
            locked: false,
            disputed_transactions: HashSet::new(),
        },
        Account {
            client: 3,
            available: 6.6,
            held: 6.9,
            total: 13.5,
            locked: false,
            disputed_transactions: HashSet::from_iter([6]),
        },
        Account {
            client: 4,
            available: 5.9,
            held: 0.0,
            total: 5.9,
            locked: false,
            disputed_transactions: HashSet::new(),
        },
    ];

    // Assert one at a time so we get more legible output
    for (o, c) in output.into_iter().zip(correct.into_iter()) {
        assert_eq!(o, c);
    }
}
