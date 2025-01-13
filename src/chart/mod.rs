use solana_sdk::pubkey::Pubkey;
use std::time::{SystemTime, UNIX_EPOCH};
pub use types::{ChartData, Timeframe};

use crate::{
    client::SolanaMirrorRpcClient,
    enums::Error,
    transactions::{get_parsed_transactions, types::ParsedTransaction},
};

pub mod types;

pub async fn get_chart_data(
    client: &SolanaMirrorRpcClient,
    address: &Pubkey,
    range: u8,
    timeframe: Timeframe,
) -> Result<Vec<ChartData>, Error> {
    let txs = get_parsed_transactions(&client, address, None).await?;
    let balance_states = get_balance_states(&txs.transactions);
    let filtered_balance_states = filter_balance_states(&balance_states, timeframe, range);

    Ok(filtered_balance_states)
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
