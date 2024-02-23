# Gem Wallet Core library in Rust

[![Tests](https://github.com/gemwalletcom/core/actions/workflows/ci.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci.yml)
[![Clippy check](https://github.com/gemwalletcom/core/actions/workflows/lint.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/lint.yml)
[![Gemstone CI](https://github.com/gemwalletcom/core/actions/workflows/ci-gemstone.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci-gemstone.yml)

## Running

### Setup Core

Run `make install` to install rust, typeshare

### Setup DB

- Setup a new role `username` and `api` database

### Chain Support

| Chain        | Transfers     | Token Transfers |
|--------------|---------------|-----------------|
| Bitcoin      | ✅            | --              |
| Litecoin     | ✅            | --              |
| Doge         | ✅            | --              |
| Ethereum     | ✅            | ✅               |
| Binance      | ✅            | ✅               |
| SmartChain   | ✅            | ✅               |
| Solana       | ✅            | ✅               |
| Polygon      | ✅            | ✅               |
| Thorchain    | ✅            | 🏗               |
| Cosmos       | ✅            | 🏗               |
| Osmosis      | ✅            | 🏗               |
| Arbitrum     | ✅            | ✅               |
| Ton          | ✅            | 🏗               |
| Tron         | ✅            | ✅               |
| Optimism     | ✅            | ✅               |
| Aptos        | ✅            | 🏗               |
| Base         | ✅            | ✅               |
| AvalancheC   | ✅            | ✅               |
| Sui          | ✅            | 🏗               |
| Xrp          | ✅            | 🏗               |
| OpBNB        | ✅            | ✅               |

List of available chains specified in [primitives package](https://github.com/gemwalletcom/core/blob/main/primitives/src/chain.rs).
