use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormattedAmount {
    pub amount: String,
    pub formatted: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FormattedAmountWithPrice {
    pub amount: FormattedAmount,
    pub price: f64,
}

#[derive(Debug)]
pub struct Page {
    pub start_idx: usize,
    pub end_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOpts {
    pub parse: bool,
}
