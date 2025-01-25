use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::FormattedAmount;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceChange {
    pub pre: FormattedAmount,
    pub post: FormattedAmount,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedTransaction {
    #[serde(rename = "blockTime")]
    pub block_time: i64,
    pub signatures: Vec<String>,
    pub balances: HashMap<String, BalanceChange>,
    #[serde(rename = "parsedInstructions")]
    pub parsed_instructions: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct TransactionResponse {
    pub transactions: Vec<ParsedTransaction>,
    pub count: usize,
}
