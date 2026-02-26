# Project Structure

Gem Wallet Core is a Rust-based cryptocurrency wallet backend engine supporting 35+ blockchain networks. It is a Cargo workspace with 50+ crates covering transaction processing, asset management, DeFi integrations, swap operations, and cross-platform mobile support.

## Directory Tree

```
apps/           # Backend services (API, Daemon, Dynode)
gemstone/       # Cross-platform mobile library (UniFFI → iOS/Android)
crates/         # Shared libraries and blockchain implementations
bin/            # Utility binaries
skills/         # Agent guidance documents (this directory)
```

## Applications (`apps/`)

- **API Server** (`apps/api/`): REST API with WebSocket price streaming
- **Daemon** (`apps/daemon/`): Background services for asset updates, push notifications, transaction indexing
- **Dynode** (`apps/dynode/`): Dynamic blockchain node proxy with caching, monitoring, and metrics

## Cross-Platform Library (`gemstone/`)

Shared Rust library compiled to iOS Swift Package and Android AAR using UniFFI bindings. Contains blockchain RPC clients, swap integrations, payment URI decoding, and message signing.
- Uses `swapper` crate via `gemstone::gem_swapper` module for on-device swap integrations
- Uses `signer` crate for cryptographic signing operations across multiple blockchain types
- **Always use UniFFI remote types for external models** — see [UniFFI Remote and External Types](https://mozilla.github.io/uniffi-rs/latest/types/remote_ext_types.html)

## Blockchain Support

Individual `gem_*` crates for each blockchain with unified RPC client patterns:
- **Bitcoin family** (`gem_bitcoin`): Bitcoin, Bitcoin Cash, Litecoin, Dogecoin
- **EVM chains** (`gem_evm`, `gem_bsc`): Ethereum, Polygon, Arbitrum, Optimism, Base, zkSync, Linea, BSC
- **Alternative L1s**: Solana (`gem_solana`), Sui (`gem_sui`), TON (`gem_ton`), Aptos (`gem_aptos`), NEAR (`gem_near`), Stellar (`gem_stellar`), Algorand (`gem_algorand`), Tron (`gem_tron`), XRP (`gem_xrp`), Cardano (`gem_cardano`), Polkadot (`gem_polkadot`)
- **Cosmos ecosystem** (`gem_cosmos`): Cosmos Hub, Osmosis, Celestia, Injective, Sei, Noble

## Utility Binaries (`bin/`)

- **uniffi-bindgen** (`bin/uniffi-bindgen/`): UniFFI bindings generator for iOS and Android
- **generate** (`bin/generate/`): Code generation utilities
- **gas-bench** (`bin/gas-bench/`): Gas benchmarking tool for blockchain operations
- **img-downloader** (`bin/img-downloader/`): Image asset downloader utility

## Core Crates

### Blockchain Infrastructure
- `gem_client/`: Client trait abstraction; implementations: `ReqwestClient` (backend) and `AlienProvider` (mobile)
- `gem_jsonrpc/`: Internal JSON-RPC client library (replaces external alloy dependencies)
- `gem_hash/`: Hashing utilities for blockchain operations
- `chain_primitives/`: Primitive types specific to blockchain operations
- `chain_traits/`: Common traits for blockchain implementations

### Cross-Chain Operations
- `swapper/`: Standalone swap/exchange integration crate supporting DEX and CEX swaps across multiple chains
- `signer/`: Cryptographic signing operations for transactions across multiple blockchain types

### Data & Storage
- `primitives/`: Central types and models shared across the system
- `storage/`: Database models, migrations, and data access layer using Diesel ORM
- `cacher/`: Caching layer for improved performance

### Pricing & Market Data
- `pricer/`: Asset pricing aggregation and management
- `prices_dex/`: DEX-specific price feeds and calculations
- `coingecko/`: CoinGecko API integration for market data

### NFT & Digital Assets
- `nft/`: NFT data models and business logic
- `nft_client/`: NFT marketplace API clients
- `nft_provider/`: NFT data provider integrations (OpenSea, Magic Eden, NFTScan)

### Integrations & Services
- `fiat/`: Fiat on-ramp/off-ramp providers (MoonPay, Transak, Mercuryo, Banxa)
- `name_resolver/`: Blockchain naming service integrations (ENS, SNS, etc.)
- `security_provider/`: Security and fraud detection provider integrations
- `api_connector/`: Backend API connector utilities
- `gem_hypercore/`: Perpetuals (perps) trading support via Hyperliquid integration

### Utilities & Support
- `localizer/`: i18n support for 20+ languages using Fluent
- `serde_serializers/`: Custom Serde serializers/deserializers used across crates
- `number_formatter/`: Number and currency formatting utilities
- `job_runner/`: Background job execution framework
- `search_index/`: Search indexing and query capabilities
- `streamer/`: Real-time data streaming utilities
- `tracing/`: Logging and tracing infrastructure
- `settings/`: Configuration management
- `settings_chain/`: Chain-specific configuration settings

## Technology Stack

- **Framework**: Rust workspace with Rocket web framework
- **Database**: PostgreSQL (primary), Redis (caching)
- **Message Queue**: RabbitMQ with Lapin
- **RPC**: Custom `gem_jsonrpc` client library for blockchain interactions
- **Mobile**: UniFFI for iOS/Android bindings
- **Serialization**: Serde with custom serializers
- **Async**: Tokio runtime
- **Testing**: Built-in Rust testing with integration tests
