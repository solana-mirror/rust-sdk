use once_cell::sync::Lazy;
use std::{collections::HashMap, str::FromStr};
use tokio::sync::Mutex;

use mpl_token_metadata::{accounts::Metadata, programs::MPL_TOKEN_METADATA_ID};
use solana_sdk::pubkey::Pubkey;

use crate::{
    balances::accounts::types::{ImageResponse, ParsedMetadata},
    client::{GetAccountDataConfig, SolanaMirrorRpcClient},
    consts::{SOL_IMAGE, USDC_IMAGE},
    enums::Error,
    types::Page,
};

pub fn clean_string(s: String) -> String {
    s.trim_matches('\0').trim_matches('"').to_string()
}

pub fn create_batches<T: Clone>(
    arr: &Vec<T>,
    batch_size: usize,
    limit: Option<u32>,
) -> Vec<Vec<T>> {
    let mut batches: Vec<Vec<T>> = Vec::new();
    let mut total_elements = 0;

    let limit = limit.map(|l| l as usize);

    for i in (0..arr.len()).step_by(batch_size) {
        let mut batch = arr[i..std::cmp::min(i + batch_size, arr.len())].to_vec();

        if let Some(limit) = limit {
            let remaining_limit = limit - total_elements;

            if batch.len() > remaining_limit {
                batch = batch[..remaining_limit].to_vec();
                batches.push(batch);
                break;
            }
        }

        total_elements += batch.len();
        batches.push(batch);
    }

    batches
}

pub fn parse_page(index: Option<(u64, u64)>) -> Result<Option<Page>, Error> {
    if index.is_none() {
        return Ok(None);
    }

    let (start_idx, end_idx) = index.unwrap();

    if end_idx < start_idx {
        return Err(Error::InvalidIndex);
    }

    Ok(Some(Page {
        start_idx: start_idx as usize,
        end_idx: end_idx as usize,
    }))
}

static METADATA_CACHE: Lazy<Mutex<HashMap<String, ParsedMetadata>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Fetches or retrieves from cache the metadata associated with the given SPL token mint address.
pub async fn fetch_metadata(client: &SolanaMirrorRpcClient, mint_address: &str) -> ParsedMetadata {
    let mut cache = METADATA_CACHE.lock().await;
    if let Some(metadata) = cache.get(mint_address) {
        return metadata.clone();
    }

    let mint_pubkey = Pubkey::from_str(mint_address).unwrap();
    let mpl_program_id = Pubkey::from_str(MPL_TOKEN_METADATA_ID.to_string().as_str()).unwrap();

    // Get the metadata account address associated with the mint
    let (metadata_pubkey, _) = Pubkey::find_program_address(
        &[
            "metadata".as_ref(),
            &mpl_program_id.to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &mpl_program_id,
    );

    let data = match client
        .get_account_info(
            &metadata_pubkey,
            Some(GetAccountDataConfig {
                commitment: None,
                encoding: Some("jsonParsed".to_string()),
            }),
        )
        .await
    {
        Ok(data) => data,
        Err(_) => return ParsedMetadata::default(),
    };

    let parsed_metadata = match Metadata::safe_deserialize(&data) {
        Ok(metadata) => parse_metadata(metadata),
        Err(_) => ParsedMetadata::default(),
    };

    cache.insert(mint_address.to_string(), parsed_metadata.clone());
    parsed_metadata
}

/// Parses the given metadata.
fn parse_metadata(metadata: Metadata) -> ParsedMetadata {
    ParsedMetadata {
        name: clean_string(metadata.name),
        symbol: clean_string(metadata.symbol),
        uri: clean_string(metadata.uri),
    }
}

static IMAGE_CACHE: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn fetch_image(metadata: &ParsedMetadata) -> String {
    let mut cache = IMAGE_CACHE.lock().await;
    if let Some(image_url) = cache.get(metadata.symbol.as_str()) {
        return image_url.to_string();
    }

    // TODO: have a more generic image fallback
    let predefined_images = HashMap::from([
        ("USDC", USDC_IMAGE),
        (
            "RCL",
            "https://ipfs.io/ipfs/Qme9ErqmQaznzpfDACncEW48NyXJPFP7HgzfoNdto9xQ9P/02.jpg",
        ),
        ("SOL", SOL_IMAGE),
    ]);

    if let Some(&url) = predefined_images.get(metadata.symbol.as_str()) {
        cache.insert(metadata.symbol.clone(), url.to_string());
        return url.to_string();
    }

    if let Ok(response) = reqwest::get(&metadata.uri).await {
        if let Ok(image_response) = response.json::<ImageResponse>().await {
            cache.insert(metadata.symbol.clone(), image_response.image.clone());
            return image_response.image;
        }
    }

    let fallback_image = String::default();
    cache.insert(metadata.symbol.clone(), fallback_image.clone());
    fallback_image
}

pub fn calculate_concentrated_liquidity_amounts(
    liquidity: u128,
    tick_lower: i32,
    tick_upper: i32,
    sqrt_price_x64: u128,
) -> (f64, f64) {
    let sqrt_price_current = get_sqrt_price_from_sqrt_price_x64(sqrt_price_x64);
    let sqrt_price_lower = get_sqrt_price_from_tick(tick_lower);
    let sqrt_price_upper = get_sqrt_price_from_tick(tick_upper);

    let liquidity_f64 = liquidity as f64;

    if sqrt_price_current <= sqrt_price_lower {
        // There is only token B (quote token)
        let amount_a =
            calculate_token_a_below_range(liquidity_f64, sqrt_price_lower, sqrt_price_upper);
        (amount_a.round(), 0.0)
    } else if sqrt_price_current < sqrt_price_upper {
        // Both tokens are present
        let (amount_a, amount_b) = calculate_tokens_in_range(
            liquidity_f64,
            sqrt_price_current,
            sqrt_price_lower,
            sqrt_price_upper,
        );
        (amount_a.round(), amount_b.round())
    } else {
        // There is only token A (base token)
        let amount_b = liquidity_f64 * (sqrt_price_upper - sqrt_price_lower);
        (0.0, amount_b.round())
    }
}

pub fn get_sqrt_price_from_tick(tick: i32) -> f64 {
    (1.0001f64.powi(tick) as f64).sqrt()
}

pub fn get_sqrt_price_from_sqrt_price_x64(sqrt_price_x64: u128) -> f64 {
    (sqrt_price_x64 as f64) / (1u128 << 64) as f64
}

/// Calculates amount of token A when price is below the range
pub fn calculate_token_a_below_range(
    liquidity: f64,
    sqrt_price_lower: f64,
    sqrt_price_upper: f64,
) -> f64 {
    liquidity * (1.0 / sqrt_price_lower - 1.0 / sqrt_price_upper)
}

/// Calculates amounts for both tokens when price is within range
pub fn calculate_tokens_in_range(
    liquidity: f64,
    sqrt_price_current: f64,
    sqrt_price_lower: f64,
    sqrt_price_upper: f64,
) -> (f64, f64) {
    let amount_a = liquidity * (1.0 / sqrt_price_current - 1.0 / sqrt_price_upper);
    let amount_b = liquidity * (sqrt_price_current - sqrt_price_lower);
    (amount_a, amount_b)
}
