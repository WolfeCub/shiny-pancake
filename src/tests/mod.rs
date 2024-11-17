use std::collections::HashSet;

use crate::csv_types::*;
use crate::engine::*;
use crate::Account;

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
fn dispute_widthdrawal() {
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
            locked: true,
            disputed_transactions: HashSet::new(),
        }]
    );
}
