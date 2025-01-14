use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::internal::FormattedAmount;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ChartData {
    pub timestamp: i64,
    pub balances: HashMap<String, FormattedAmount>,
}

#[derive(Debug)]
pub enum Timeframe {
    Hour,
    Day,
}

impl Timeframe {
    pub fn new(timeframe: &str) -> Option<Self> {
        match timeframe.to_lowercase().as_str() {
            "h" => Some(Self::Hour),
            "d" => Some(Self::Day),
            _ => None,
        }
    }

    pub fn to_seconds(timeframe: Self) -> i64 {
        match timeframe {
            Self::Hour => 3600,
            Self::Day => 86400,
        }
    }
}
