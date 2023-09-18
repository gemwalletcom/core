# core
![Tests](https://github.com/gemwalletcom/core/workflows/Tests/badge.svg)

## Running

### Setup Core

Run `make install` to install rust, typeshare

### Setup API Env

- Install Postgres and redis `brew install postgresql@14 redis`
- - Setup a new `username` + `api` database
- Install diesel.rs `cargo install diesel_cli --no-default-features --features postgres`

### Chain Support

| Chain        | Transfers     | 
|--------------|---------------|
| Bitcoin      | 🏗            |
| Ethereum     | ✅            |
| Binance      | ✅            |
| SmartChain   | ✅            |
| Solana       | ✅            |
| Polygon      | ✅            |
| Thorchain    | ✅            |
| Cosmos       | ✅            |
| Osmosis      | ✅            |
| Arbitrum     | ✅            |
| Ton          | ✅            |
| Tron         | ✅            |
| Doge         | 🏗            |
| Optimism     | ✅            |
| Aptos        | 🏗            |
| Base         | ✅            |
| AvalancheC   | ✅            |
| Sui          | 🏗            |
| Ripple       | ✅            |
| OpBNB        | ✅            |

List of available chains specified in [primitives package](https://github.com/gemwalletcom/core/blob/main/primitives/src/chain.rs).