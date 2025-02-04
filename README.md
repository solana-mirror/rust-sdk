# SolanaMirror Rust SDK

Rust SDK that helps fetch token balances, transaction history, and historical chart data for a Solana wallet.

## Getting Started

First, initialize the client:

```rust
let client = solana_mirror::SolanaMirror::new(watch, rpc);
```

Params:

- `watch`: The address to fetch.
- `rpc`: Solana RPC client.

## Fetching Token Accounts

Get all associated token accounts (ATAs) for a wallet, including positions in dapps like Raydium:

```rust
let token_accounts = client.get_token_accounts(Some(true)).await?;
```

Params:

- `show_apps`: Boolean flag to include positions in the response.

Returns:

- `Vec<ParsedAta>`: List of token accounts with metadata and balances.

- `Option<Vec<ParsedPosition>>`: Optional list of positions in liquidity pools.

## Fetching Transactions

Retrieve transactions for a wallet, with balances before and after each transaction:

```rust
let transactions = client.get_transactions(Some((0, 10))).await?;
```

Params:

- `(start, end)`: Pagination index range.

Returns:

- `TransactionResponse`: Contains a list of parsed transactions.

## Fetching Chart Data

Get historical balances over a specified timeframe:

```rust
    let chart_data = client.get_chart_data(14, Timeframe::Day).await?;
```

Params:

- `range`: Number of time periods to include.

- `timeframe`: Either Daily or Hourly.

Returns:

- `Vec<ChartData>`: Reconstructed historical token balances.