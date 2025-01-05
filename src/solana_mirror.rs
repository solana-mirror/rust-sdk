use std::sync::Arc;

use crate::balances::accounts::{get_parsed_accounts, ParsedAta};
use crate::balances::dapps::raydium::{get_raydium_positions, ParsedPosition};
use crate::chart::{get_chart_data, ChartResponse, Timeframe};
use crate::client::SolanaMirrorRpcClient;
use crate::coingecko::CoingeckoClient;
use crate::enums::Error;
use crate::transactions::{get_parsed_transactions, TransactionResponse};
use reqwest::Client;
use solana_sdk::pubkey::Pubkey;

/// Main struct for interacting with Solana Mirror SDK
pub struct SolanaMirror {
    /// The Solana address being watched
    watch: Pubkey,
    /// Client instance for making RPC calls
    client: SolanaMirrorRpcClient,
    /// Coingecko instance for getting historical token prices
    coingecko_client: CoingeckoClient,
}

impl SolanaMirror {
    /// Creates a new instance of SolanaMirror
    ///
    /// # Arguments
    /// * `watch` - The Solana address to watch
    pub fn new(watch: Pubkey, rpc_url: String) -> Self {
        let http_client = Arc::new(Client::new());

        Self {
            watch,
            client: SolanaMirrorRpcClient::new(http_client.clone(), rpc_url),
            coingecko_client: CoingeckoClient::new(http_client),
        }
    }

    /// Returns the address being watched
    pub fn get_watch_address(&self) -> Pubkey {
        self.watch
    }

    /// Changes the address being watched
    ///
    /// # Arguments
    /// * `address` - The new address to watch
    pub fn set_watch_address(&mut self, address: Pubkey) {
        self.watch = address;
    }

    /// Gets the token accounts and positions for the watched address
    ///
    /// # Arguments
    /// * `show_apps` - Whether to include dapp positions
    /// * `opts` - Optional fetch configuration
    pub async fn get_token_accounts(
        &self,
        show_apps: Option<bool>,
    ) -> Result<(Vec<ParsedAta>, Option<Vec<ParsedPosition>>), Error> {
        let accounts = get_parsed_accounts(&self.client, &self.watch).await?;

        let positions = if show_apps.unwrap_or(false) {
            Some(get_raydium_positions(&self.client, &self.watch).await?)
        } else {
            None
        };

        Ok((accounts, positions))
    }

    /// Fetches and parses transactions for the watched address
    ///
    /// # Arguments
    /// * `index` - Optional tuple of [start, end] for pagination
    /// * `opts` - Optional fetch configuration
    pub async fn get_transactions(
        &self,
        index: Option<(u64, u64)>,
    ) -> Result<TransactionResponse, Error> {
        get_parsed_transactions(&self.client, &self.watch, index).await
    }

    /// Fetches transaction history and returns chart data
    ///
    /// # Arguments
    /// * `range` - Number of time periods to include
    /// * `timeframe` - Either Daily or Hourly
    /// * `detailed` - Whether to include detailed state information
    /// * `opts` - Optional fetch configuration
    pub async fn get_chart_data(
        &self,
        range: u8,
        timeframe: Timeframe,
        detailed: Option<bool>,
    ) -> Result<ChartResponse, Error> {
        get_chart_data(
            &self.client,
            &self.coingecko_client,
            &self.watch,
            range,
            timeframe,
            detailed,
        )
        .await
    }
}
