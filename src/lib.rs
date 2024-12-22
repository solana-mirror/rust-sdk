//! # Solana Mirror
//! 
//! A Rust library for retrieving current and historical data for a given Solana wallet
//! 
//! ## Features
//! 
//! - Current token and positions balance
//! - Transaction history
//! - Chart data

pub mod balances;
pub mod chart;
pub mod client;
pub mod coingecko;
pub mod price;
pub mod transactions;
pub mod types;
pub mod utils; 

pub const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const SOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const USDC_IMAGE: &str = "https://s2.coinmarketcap.com/static/img/coins/128x128/3408.png";
pub const SOL_IMAGE: &str = "https://s2.coinmarketcap.com/static/img/coins/128x128/5426.png";

#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidIndex,
    InvalidTimeframe,
    FetchError,
    ParseError,
    TooManyRequests,
}

#[derive(Debug)]
pub struct Page {
    pub start_idx: usize,
    pub end_idx: usize,
}
