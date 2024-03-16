# Gem Wallet Core library in Rust

[![Tests](https://github.com/gemwalletcom/core/actions/workflows/ci.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci.yml)
[![Clippy check](https://github.com/gemwalletcom/core/actions/workflows/lint.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/lint.yml)
[![Gemstone CI](https://github.com/gemwalletcom/core/actions/workflows/ci-gemstone.yml/badge.svg)](https://github.com/gemwalletcom/core/actions/workflows/ci-gemstone.yml)

# Gem Core Library

The Gem Core Library is a Rust monorepo utilized within the Gem wallet. The Android and iOS versions of the wallet utilize this core as backend infrastructure, performing tasks such as push notifications, pricing, assets information, and more.

## Gemstone 

The Gemstone is a cross-platform library for Swift and Kotlin.

## Cryptography

The cryptography for the Gem wallet is implemented using [trust-wallet-core](https://github.com/trustwallet/wallet-core).

```mermaid
flowchart graph
    client[gem client (swift/kotlin)]
    core-lib[gem wallet core library monorepo]
    trust-core[trust wallet core]
    client-- pricing, notification and ... -->core-lib
    client-- cryptography such as seed generation and ... -->trust-core
```

> Note: According to the roadmap, in the future, the client will only interact with the Gem Wallet Core, and the interaction with the Trust Wallet Core will be the responsibility of the Gem Wallet Core.

## Running

### Setup Core

Run `make install` to install rust, typeshare

### Setup DB

- Create a new database `api` and grant privileges to `username` role
- Run `diesel migration run` to create tables and do migrations

### Supported Chains

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
