use std::collections::HashSet;

use crate::csv_types::*;
use crate::engine::*;
use crate::Account;

#[test]
fn simple_deposits() {
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(5.0)));

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(5.0)));

    engine.process(CsvRow::new(CsvTransaction::Deposit, 2, 3, Some(7.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 2, 4, Some(11.0)));

    let mut output = engine.output();
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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Withdrawal, 1, 2, Some(6.0)));

    let output = engine.output();

    assert_eq!(output, vec![Account::new(1)]);
}

#[test]
fn simple_dispute() {
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Dispute, 1, 1, None));

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Dispute, 1, 1, None));
    engine.process(CsvRow::new(CsvTransaction::Resolve, 1, 1, None));

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Dispute, 1, 1, None));
    engine.process(CsvRow::new(CsvTransaction::Chargeback, 1, 1, None));

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Resolve, 1, 1, None));

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Chargeback, 1, 1, None));

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(5.0)));
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 2, Some(1.0)));
    engine.process(CsvRow::new(CsvTransaction::Dispute, 1, 1, None));
    engine.process(CsvRow::new(CsvTransaction::Resolve, 1, 1, None));
    engine.process(CsvRow::new(CsvTransaction::Resolve, 1, 1, None)); // We resolve it again but it
                                                                      // shouldn't be under disput
                                                                      // anymore

    let output = engine.output();

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
    let mut engine = Engine::new();
    engine.process(CsvRow::new(CsvTransaction::Deposit, 1, 1, Some(10.0)));
    engine.process(CsvRow::new(CsvTransaction::Withdrawal, 1, 2, Some(3.0)));
    engine.process(CsvRow::new(CsvTransaction::Dispute, 1, 2, None));

    let output = engine.output();

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
