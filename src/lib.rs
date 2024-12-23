//! # Solana Mirror
//!
//! A Rust library for retrieving current and historical data for a given Solana wallet
//!
//! ## Features
//!
//! - Current token balances and open positions
//! - Transaction history
//! - Chart data

mod balances;
mod chart;
mod client;
mod coingecko;
mod consts;
mod enums;
mod price;
mod transactions;
mod types;
mod utils;

// TODO: Create a SolanaMirrorSDK class to handle Coingecko and RPC clients dynamically, instead of declaring them inside each function

pub use client::SolanaMirrorClient;
pub use utils::get_rpc;

pub use balances::accounts::get_parsed_accounts;
pub use balances::dapps::raydium::get_raydium_positions;
pub use chart::get_chart_data;
pub use transactions::get_parsed_transactions;
