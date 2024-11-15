use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CsvTransaction {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvRow {
    #[serde(rename = "type")]
    pub transaction_type: CsvTransaction,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
}

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

