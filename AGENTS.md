# AGENTS.md

Guidance for AI assistants (Claude Code, Gemini, Codex, etc.) collaborating on this repository.

## Purpose & Audience

This document orients coding agents to the repo structure, development workflow, coding standards, and core architectural patterns used across Gem Wallet Core.

## Project Overview

Gem Wallet Core is a Rust-based cryptocurrency wallet backend engine supporting 35+ blockchain networks. It is a Cargo workspace with 50+ crates covering transaction processing, asset management, DeFi integrations, swap operations, and cross-platform mobile support.

## Repository Layout

### Applications (`apps/`)
- **API Server** (`apps/api/`): REST API with WebSocket price streaming
- **Daemon** (`apps/daemon/`): Background services for asset updates, push notifications, transaction indexing
- **Dynode** (`apps/dynode/`): Dynamic blockchain node proxy with caching, monitoring, and metrics

### Cross-Platform Library (`gemstone/`)
Shared Rust library compiled to iOS Swift Package and Android AAR using UniFFI bindings. Contains blockchain RPC clients, swap integrations, payment URI decoding, and message signing. Uses the separate `swapper` and `signer` crates for swap and signing operations.
- Uses `swapper` crate via `gemstone::gem_swapper` module for on-device swap integrations
- Uses `signer` crate for cryptographic signing operations across multiple blockchain types
- **Always use UniFFI remote types for external models** - See [UniFFI Remote and External Types](https://mozilla.github.io/uniffi-rs/latest/types/remote_ext_types.html) for proper integration patterns

### Blockchain Support
Individual `gem_*` crates for each blockchain with unified RPC client patterns:
- **Bitcoin family** (`gem_bitcoin`): Bitcoin, Bitcoin Cash, Litecoin, Dogecoin
- **EVM chains** (`gem_evm`, `gem_bsc`): Ethereum, Polygon, Arbitrum, Optimism, Base, zkSync, Linea, BSC
- **Alternative L1s**: Solana (`gem_solana`), Sui (`gem_sui`), TON (`gem_ton`), Aptos (`gem_aptos`), NEAR (`gem_near`), Stellar (`gem_stellar`), Algorand (`gem_algorand`), Tron (`gem_tron`), XRP (`gem_xrp`), Cardano (`gem_cardano`), Polkadot (`gem_polkadot`)
- **Cosmos ecosystem** (`gem_cosmos`): Cosmos Hub, Osmosis, Celestia, Injective, Sei, Noble

### Utility Binaries (`bin/`)
- **uniffi-bindgen** (`bin/uniffi-bindgen/`): UniFFI bindings generator for iOS and Android
- **generate** (`bin/generate/`): Code generation utilities
- **gas-bench** (`bin/gas-bench/`): Gas benchmarking tool for blockchain operations
- **img-downloader** (`bin/img-downloader/`): Image asset downloader utility

### Core Services & Infrastructure Crates

#### Blockchain Infrastructure
- `gem_client/`: Client trait abstraction used across services; implementations: `ReqwestClient` (backend) and `AlienProvider` (mobile)
- `gem_jsonrpc/`: Internal JSON-RPC client library (replaces external alloy dependencies)
- `gem_hash/`: Hashing utilities for blockchain operations
- `chain_primitives/`: Primitive types specific to blockchain operations
- `chain_traits/`: Common traits for blockchain implementations

#### Cross-Chain Operations
- `swapper/`: Standalone swap/exchange integration crate supporting DEX and CEX swaps across multiple chains
- `signer/`: Cryptographic signing operations for transactions across multiple blockchain types

#### Data & Storage
- `primitives/`: Central types and models shared across the system
- `storage/`: Database models, migrations, and data access layer using Diesel ORM
- `cacher/`: Caching layer for improved performance

#### Pricing & Market Data
- `pricer/`: Asset pricing aggregation and management
- `prices_dex/`: DEX-specific price feeds and calculations
- `coingecko/`: CoinGecko API integration for market data

#### NFT & Digital Assets
- `nft/`: NFT data models and business logic
- `nft_client/`: NFT marketplace API clients
- `nft_provider/`: NFT data provider integrations (OpenSea, Magic Eden, NFTScan)

#### Integrations & Services
- `fiat/`: Fiat on-ramp/off-ramp providers (MoonPay, Transak, Mercuryo, Banxa)
- `name_resolver/`: Blockchain naming service integrations (ENS, SNS, etc.)
- `security_provider/`: Security and fraud detection provider integrations
- `api_connector/`: Backend API connector utilities
- `gem_hypercore/`: Perpetuals (perps) trading support via Hyperliquid integration

#### Utilities & Support
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

- Framework: Rust workspace with Rocket web framework
- Database: PostgreSQL (primary), Redis (caching)
- Message Queue: RabbitMQ with Lapin
- RPC: Custom `gem_jsonrpc` client library for blockchain interactions
- Mobile: UniFFI for iOS/Android bindings
- Serialization: Serde with custom serializers
- Async: Tokio runtime
- Testing: Built-in Rust testing with integration tests

## Development Workflow

All commands use the `just` task runner. Run from the workspace root unless specified.

### Build
- `just build`: Build the workspace
- `just build-gemstone`: Build cross-platform library
- `just gemstone build-ios`: Build iOS Swift Package (run in `gemstone/`)
- `just gemstone build-android`: Build Android AAR (run in `gemstone/`)

### Test
- `just test-workspace`: Run all workspace tests
- `just test-all`: Run all tests including integration
- `just test <CRATE>`: Test a specific crate
- `just gemstone test-ios`: Run iOS integration tests (run in `gemstone/`)
- `cargo test --test integration_test --package <CRATE> --features <FEATURE>`: Run integration tests manually

### Code Quality
- `just format`: Format all code
- `just lint`: Run clippy with warnings as errors
- `just fix`: Auto-fix clippy issues
- `just unused`: Find unused dependencies with cargo-machete

### Database
- `just migrate`: Run Diesel migrations
- `just setup-services`: Start Docker services (PostgreSQL, Redis, Meilisearch, RabbitMQ)

### Mobile
- `just gemstone install-ios-targets`: Install iOS Rust targets (run in `gemstone/`)
- `just gemstone install-android-targets`: Install Android Rust targets and `cargo-ndk` (run in `gemstone/`)
- Note: Mobile builds require UniFFI bindings generation and platform-specific compilation

### Generating Bindings (After Core Code Changes)
**IMPORTANT**: When you modify code in `gemstone/`, `swapper/`, `signer/`, or any crate that affects the mobile API, you MUST regenerate the platform bindings.

#### Swift Bindings (iOS)
- `just gemstone bindgen-swift`: Generate Swift bindings only (run in `gemstone/`)
- `just gemstone build-ios`: Full iOS build including Swift binding generation (run in `gemstone/`)
- Generated files location: `gemstone/generated/swift/` and copied to `gemstone/target/spm/`

#### Kotlin Bindings (Android)
- `just gemstone bindgen-kotlin`: Generate Kotlin bindings only (run in `gemstone/`)
- `just gemstone build-android`: Full Android build including Kotlin binding generation (run in `gemstone/`)
- Generated files location: `gemstone/generated/kotlin/` and copied to `gemstone/android/gemstone/src/main/java/uniffi/`

#### When to Regenerate Bindings
1. After adding/modifying public functions in `gemstone/src/lib.rs`
2. After changing any UniFFI-exposed types or interfaces
3. After modifying `swapper/` or `signer/` crates that are exposed via gemstone
4. Before committing changes that affect the mobile API surface
5. When UniFFI schema or configuration changes

### Utilities
- `just localize`: Update localization files
- `just generate-ts-primitives`: Generate TypeScript types from Rust
- `just outdated`: Check for outdated dependencies

## Coding Standards

Follow the existing code style patterns unless explicitly asked to change

### Code Formatting
- Line length: 160 characters maximum (configured in `rustfmt.toml`)
- Indentation: 4 spaces (Rust standard)
- Imports: Automatically reordered with rustfmt
 - ALWAYS run `just format` before committing
 - Formatter enforces consistent style across all crates/workspace

### Commit Messages
- Write descriptive messages following conventional commit format

### Naming and Conventions
- Files/modules: `snake_case` (e.g., `asset_id.rs`, `chain_address.rs`)
- Crates: Prefixed naming (`gem_*` for blockchains, `security_*` for security)
- Functions/variables: `snake_case`
- Structs/enums: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Helper names: inside a module stick to concise names that rely on scope rather than repeating crate/module prefixes (e.g., prefer `is_spot_swap` over `is_hypercore_spot_swap` in `core_signer.rs`).
- Don't use `util`, `utils`, `normalize`, or any other similar names for modules or functions.
- Avoid using `matches!` for pattern matching as much as possible, it's easy to missing a case later.

### Imports
1. Standard library imports first
2. External crate imports
3. Local crate imports
4. Module re-exports with `pub use`

IMPORTANT: Always import models and types at the top of the file. Never use inline imports inside functions (e.g., `use crate::models::SomeType` inside a function). Declare all imports in the file header.

### Error Handling
- Use `thiserror` for custom error types
- Implement `From` traits for error conversion
- Use consistent `Result<T, Error>` return types
- Propagate errors with the `?` operator
- Add smart `From` conversions (e.g., `From<serde_json::Error> for SignerError`) so callers can prefer `?` over manual `map_err`.

### Database Patterns
- Separate database models from domain primitives
- Use `as_primitive()` methods for conversion
- Diesel ORM with PostgreSQL backend
- Support transactions and upserts

### Async Patterns
- Tokio runtime throughout
- Async client structs returning `Result<T, Error>`
- Use `Arc<tokio::sync::Mutex<T>>` for shared async state

## Architecture & Patterns

### Key Development Patterns
- One crate per blockchain using unified RPC client patterns
- UniFFI bindings require careful Rust API design for mobile compatibility
- Use `BigDecimal` for financial precision
- Use async/await with Tokio across services
- Database models use Diesel ORM with automatic migrations
- Consider cross-platform performance constraints for mobile

### Repository Pattern

Services access repositories through direct methods on `DatabaseClient`. This pattern:
- Separates data access and business logic
- Assigns each repository a specific domain (assets, devices, etc.)
- Implements all repository traits directly on `DatabaseClient`
- Returns primitive types from repository methods, not database models
- Simplifies the API via direct method calls

Example:
```rust
pub struct AssetsClient {
    database: Box<DatabaseClient>,
}

impl AssetsClient {
    pub fn new(database_url: &str) -> Self {
        let database = Box::new(DatabaseClient::new(database_url));
        Self { database }
    }
    
    pub fn get_asset(&mut self, id: &str) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.database.assets().get_asset(id)
    }
    
    pub fn get_assets_by_device_id(&mut self, device_id: &str) -> Result<Vec<Asset>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.subscriptions().get_subscriptions_by_device_id(device_id)?;
        // ... process subscriptions
        self.database.assets().get_assets(asset_ids)
    }
}
```

Direct repository access methods available on `DatabaseClient` include:
- `assets()` - Asset operations
- `devices()` - Device operations
- `subscriptions()` - Subscription operations
- `prices()` - Price operations
- `transactions()` - Transaction operations
- And more...

### RPC Client Patterns
- Use `gem_jsonrpc::JsonRpcClient` for blockchain RPC interactions
- Prefer `alloy_primitives::hex::encode_prefixed()` for hex encoding with `0x` prefix
- **Always use `alloy_primitives::hex::decode()` for hex decoding** - it handles `0x` prefix automatically
- Use `alloy_primitives::Address::to_string()` instead of manual formatting
- RPC calls expect hex strings directly; avoid double encoding
- Use `JsonRpcClient::batch_call()` for batch operations
- Propagate errors via `JsonRpcError`

### Blockchain Provider Patterns
- Each blockchain crate has a `provider/` directory with trait implementations
- Provider methods should fetch raw data via RPC, then call mapper functions for conversion
- Place mapper functions in separate `*_mapper.rs` files for clean separation
- Example: `get_balance_coin()` calls `self.get_balance()` then `balances_mapper::map_coin_balance()`
- This pattern ensures consistent data transformation and testability across all blockchain implementations

## Testing

### Conventions
- Place integration tests in `tests/` directories
- Use `#[tokio::test]` for async tests
- Prefix test names descriptively with `test_`
- Use `Result<(), Box<dyn std::error::Error + Send + Sync>>` for test error handling
- Configure integration tests with `test = false` and appropriate `required-features` for manual execution
- Prefer real networks for RPC client tests (e.g., Ethereum mainnet)
- Test data management: For long JSON test data (>20 lines), store in `testdata/` and load with `include_str!()`; per-crate layout is typically `src/`, `tests/`, `testdata/`

### Integration Testing
- Add integration tests for RPC functionality to verify real network compatibility
- Prefer recent blocks for batch operations (more reliable than historical blocks)
- Verify both successful calls and proper error propagation
- Use realistic contract addresses (e.g., USDC) for `eth_call` testing
