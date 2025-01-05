use dotenv::dotenv;
use rocket::tokio;
use solana_mirror::{chart::Timeframe, utils::get_rpc, SolanaMirror};
use solana_sdk::pubkey::Pubkey;

const TEST_ACCOUNT: &str = "C5CsUVrjNzcpqjTGfZc8PBtyMa4PTAjPpmpM1MAP1dtH";

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = SolanaMirror::new(Pubkey::from_str_const(TEST_ACCOUNT), get_rpc());
    let chart_data = client.get_chart_data(89, Timeframe::Day, Some(true)).await;

    println!("{:?}", chart_data);
}
