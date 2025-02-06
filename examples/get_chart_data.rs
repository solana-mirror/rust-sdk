use dotenv::dotenv;
use solana_mirror::{self, ChartData, Timeframe};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use tokio::main;

#[main]
async fn main() {
    dotenv().ok();

    let test_address =
        std::env::var("TEST_ADDRESS").expect("TEST_ADDRESS environment variable not set");

    let watch = Pubkey::from_str(&test_address).expect("Invalid public key format");
    let rpc = std::env::var("RPC").expect("RPC environment variable not set");

    let client = solana_mirror::SolanaMirror::new(watch, rpc);

    let chart_data: Vec<ChartData> = client.get_chart_data(14, Timeframe::Day).await.unwrap();

    println!("{:?}", chart_data);
}
