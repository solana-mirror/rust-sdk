use reqwest::Client;
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use types::{
    ChartData, ChartDataWithPrice, ChartResponse, GetCoinMarketChartParams, MinimalChartData,
};

use crate::{
    client::SolanaMirrorClient,
    coingecko::{get_coingecko_id, CoingeckoClient},
    consts::SOL_ADDRESS,
    enums::Error,
    get_rpc,
    price::get_price,
    transactions::{get_parsed_transactions, types::ParsedTransaction},
    types::FormattedAmountWithPrice,
};

#[derive(Debug)]
pub enum Timeframe {
    Hour,
    Day,
}

pub mod types;

impl Timeframe {
    pub fn new(timeframe: &str) -> Option<Self> {
        match timeframe.to_lowercase().as_str() {
            "h" => Some(Self::Hour),
            "d" => Some(Self::Day),
            _ => None,
        }
    }

    #[allow(dead_code)]
    // TODO: can remove this
    pub fn to_string(timeframe: Self) -> String {
        match timeframe {
            Self::Hour => String::from("h"),
            Self::Day => String::from("d"),
        }
    }

    pub fn to_seconds(timeframe: Self) -> i64 {
        match timeframe {
            Self::Hour => 3600,
            Self::Day => 86400,
        }
    }
}

pub async fn get_chart_data(
    address: &str,
    timeframe: &str,
    detailed: Option<bool>,
) -> Result<ChartResponse, Error> {
    let timeframe_str = &timeframe[timeframe.len() - 1..];
    let parsed_timeframe = match Timeframe::new(timeframe_str) {
        Some(parsed_timeframe) => parsed_timeframe,
        None => return Err(Error::InvalidTimeframe),
    };
    // Gets the rest of the timeframe string (the amount of hours/days)
    let range = match timeframe[..timeframe.len() - 1].parse::<u8>() {
        Ok(range) => {
            if timeframe_str.to_lowercase() == "h" && range > 24 * 90 {
                return Err(Error::InvalidTimeframe);
            }
            range
        }
        Err(_) => return Err(Error::InvalidTimeframe),
    };

    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Error::InvalidAddress),
    };

    let reqwest = Client::new();
    let coingecko = CoingeckoClient::from_client(&reqwest);
    let client = SolanaMirrorClient::from_client(&reqwest, get_rpc());

    let txs = get_parsed_transactions(&client, &pubkey, None).await?;
    let states = get_balance_states(&txs.transactions);
    let filtered_states = filter_balance_states(&states, parsed_timeframe, range);
    let chart_data = get_price_states(&client, &coingecko, &filtered_states).await?;

    if detailed.unwrap_or(false) {
        Ok(ChartResponse::Detailed(chart_data))
    } else {
        let minimal_chart_data: Vec<MinimalChartData> = chart_data
            .iter()
            .map(|x| MinimalChartData {
                timestamp: x.timestamp,
                usd_value: x.usd_value,
            })
            .collect();

        Ok(ChartResponse::Minimal(minimal_chart_data))
    }
}

/// Creates a series of states with the balances of a wallet at each transaction
fn get_balance_states(txs: &Vec<ParsedTransaction>) -> Vec<ChartData> {
    let mut states: Vec<ChartData> = Vec::with_capacity(txs.len());

    for tx in txs {
        let mut state = ChartData {
            timestamp: tx.block_time,
            balances: states
                .last()
                .map_or(Default::default(), |last_state| last_state.balances.clone()),
        };

        for (mint, formatted_balance) in &tx.balances {
            if formatted_balance.post.formatted == 0.0 {
                state.balances.remove(mint);
            } else {
                state
                    .balances
                    .insert(mint.to_string(), formatted_balance.post.clone());
            }
        }

        states.push(state);
    }

    states
}

fn filter_balance_states(
    states: &Vec<ChartData>,
    timeframe: Timeframe,
    range: u8,
) -> Vec<ChartData> {
    if states.is_empty() {
        return Vec::new();
    }

    let t_seconds = Timeframe::to_seconds(timeframe);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let oldest_tx_timestamp = states[0].timestamp;
    let wallet_age = ((now - oldest_tx_timestamp) as f64 / t_seconds as f64).ceil() as u8;

    let adjusted_range = std::cmp::min(range, wallet_age);

    let mut filtered_states: Vec<ChartData> = Vec::new();

    let final_t = (now as f64 / t_seconds as f64).floor() as i64 * t_seconds;
    let initial_t = final_t - (adjusted_range as i64 * t_seconds);

    let mut last_idx = 0;

    for i in 0..adjusted_range {
        let t = initial_t + (i as i64 * t_seconds);

        for j in last_idx..states.len() {
            if states[j].timestamp >= t {
                let state_to_use = if j > 0 { &states[j - 1] } else { &states[j] };

                filtered_states.push(ChartData {
                    timestamp: t,
                    balances: state_to_use.balances.clone(),
                });

                last_idx = j;
                break;
            }
        }

        // Fill empty periods (eg. last state was 5 days ago, copy that state until today)
        // The main use of this is being able to fill the right price in
        if let Some(last_state) = filtered_states.last() {
            if last_state.timestamp != t {
                filtered_states.push(ChartData {
                    timestamp: t,
                    ..last_state.clone()
                })
            }
        }
    }

    // Push one last state with the current timestamp
    // in case the balance changes from day start to present
    if let Some(last_state) = filtered_states.last() {
        filtered_states.push(ChartData {
            timestamp: now,
            ..last_state.clone()
        })
    }

    filtered_states
}

async fn get_price_states(
    client: &SolanaMirrorClient,
    coingecko_client: &CoingeckoClient,
    states: &Vec<ChartData>,
) -> Result<Vec<ChartDataWithPrice>, Error> {
    let mut coingecko_prices: HashMap<String, Vec<(u64, f64)>> = HashMap::new();

    let unique_mints: HashSet<String> = states
        .iter()
        .flat_map(|state| state.balances.keys())
        .cloned()
        .collect();

    let from = states.first().map_or(0, |state| state.timestamp);
    let to = states.last().map_or(0, |state| state.timestamp);
    let diff_d = (to - from) / 86400;
    // Handle edge case in which coingecko returns daily data (more than 90 days)
    // The states can't come in hourly if they represent more than 90 days, the API returns 400
    let time_step = if diff_d > 90 { 86400 } else { 3600 };

    // Save the coingecko prices for each unique mint
    // From == to means there's no need for Coingecko
    if from != to {
        for mint in &unique_mints {
            if let Some(id) = get_coingecko_id(mint).await {
                let params = GetCoinMarketChartParams {
                    id,
                    vs_currency: "usd".to_string(),
                    from,
                    to,
                };

                if let Ok(prices) = coingecko_client.get_coin_market_chart(params).await {
                    coingecko_prices.insert(mint.clone(), prices);
                }
            }
        }
    }

    let mut new_states: Vec<ChartDataWithPrice> = Vec::with_capacity(states.len());
    let last_state_index = states.len() - 1;

    for (i, state) in states.into_iter().enumerate() {
        let timestamp = state.timestamp;
        let mut bals_with_price = HashMap::new();

        for (mint, balance) in &state.balances {
            let price = if i == last_state_index {
                // Get current price from Jup for accurracy
                let decimals = if mint == SOL_ADDRESS { Some(9) } else { None };
                get_price(client, Pubkey::from_str(mint).unwrap(), decimals)
                    .await
                    .unwrap_or(0.0)
            } else {
                // Get the right index from the Coingecko prices
                let index = ((timestamp - from) / time_step) as usize;
                coingecko_prices
                    .get(mint)
                    .and_then(|prices| prices.get(index))
                    .map_or(0.0, |(_, p)| *p)
            };

            bals_with_price.insert(
                mint.clone(),
                FormattedAmountWithPrice {
                    amount: balance.clone(),
                    price,
                },
            );
        }

        let usd_value = bals_with_price
            .values()
            .map(|b| b.amount.formatted * b.price)
            .sum();

        new_states.push(ChartDataWithPrice {
            timestamp,
            balances: bals_with_price,
            usd_value,
        });
    }

    Ok(new_states)
}
