# CLAUDE.md

This file provides guidance to AI assistants (Claude Code, Gemini, Codex, etc.) when working with code in this repository.

## Project Overview

Gem Wallet Core is a Rust-based cryptocurrency wallet backend engine supporting 35+ blockchain networks. It's structured as a Cargo workspace with 40+ crates providing comprehensive wallet functionality including transaction processing, asset management, DeFi integrations, and cross-platform mobile support.

## Architecture

### Core Applications (`apps/`)
- **API Server** (`apps/api/`) - REST API with WebSocket price streaming
- **Daemon** (`apps/daemon/`) - Background services for asset updates, push notifications, transaction indexing
- **Parser** (`apps/parser/`) - Multi-chain transaction parsing with message queue integration
- **Setup** (`apps/setup/`) - Database initialization

### Cross-Platform Library (`gemstone/`)
Shared Rust library compiled to iOS Swift Package and Android AAR using UniFFI bindings. Contains blockchain RPC clients, swap integrations, payment URI decoding, and message signing.

### Blockchain Support
Individual `gem_*` crates for each blockchain with unified RPC client patterns:
- Bitcoin family: Bitcoin, Bitcoin Cash, Litecoin, Dogecoin
- Ethereum & L2s: Ethereum, Polygon, Arbitrum, Optimism, Base, zkSync, Linea
- Alternative L1s: Solana, Sui, TON, Aptos, NEAR, Stellar, Algorand
- Cosmos ecosystem: Cosmos Hub, Osmosis, Celestia, Injective, Sei, Noble

### Core Services
- `primitives/` - Central types and models shared across the system
- `storage/` - Database models, migrations, and data access layer using Diesel ORM
- `name_resolver/` - ENS, SNS, and other naming service integrations
- `gem_evm/` - EVM blockchain support with unified RPC client patterns
- `gem_jsonrpc/` - Internal JSON-RPC client library (replaces external alloy dependencies)
- `fiat/` - Integration with fiat providers (MoonPay, Transak, Mercuryo, Banxa)
- `nft/` - NFT marketplace integrations (OpenSea, Magic Eden, NFTScan)
- `pricer/` - Asset pricing from CoinGecko and DEX sources
- `localizer/` - i18n support for 20+ languages using Fluent

## Development Commands

All commands use `just` task runner:

### Building
- `just build` - Build workspace
- `just build-gemstone` - Build cross-platform library
- `just gemstone build-ios` - Build iOS Swift Package (in gemstone/)
- `just gemstone build-android` - Build Android AAR (in gemstone/)

### Testing
- `just test-workspace` - Run all workspace tests
- `just test-all` - Run all tests including integration
- `just test <CRATE>` - Test specific crate
- `just gemstone test-ios` - Run iOS integration tests (in gemstone/)
- `cargo test --test integration_test --package <CRATE> --features <FEATURE>` - Run integration tests manually

### Code Quality
- `just format` - Format all code
- `just lint` - Run clippy with warnings as errors
- `just fix` - Auto-fix clippy issues
- `just unused` - Find unused dependencies with cargo-machete

### Database
- `just migrate` - Run Diesel migrations
- `just setup-services` - Start Docker services (PostgreSQL, Redis, Meilisearch, RabbitMQ)

### Mobile Development
- `just gemstone install-ios-targets` - Install iOS Rust targets (in gemstone/)
- `just gemstone install-android-targets` - Install Android Rust targets and cargo-ndk (in gemstone/)
- Mobile builds require UniFFI bindings generation and platform-specific compilation

### Utilities
- `just localize` - Update localization files
- `just generate-ts-primitives` - Generate TypeScript types from Rust
- `just outdated` - Check for outdated dependencies

## Technology Stack

- **Framework**: Rust workspace with Rocket web framework
- **Database**: PostgreSQL (primary), Redis (caching)
- **Message Queue**: RabbitMQ with Lapin
- **RPC**: Custom `gem_jsonrpc` client library for blockchain interactions
- **Mobile**: UniFFI for iOS/Android bindings
- **Serialization**: Serde with custom serializers
- **Async**: Tokio runtime
- **Testing**: Built-in Rust testing with integration tests

## Coding Style

Follow the existing code style patterns unless explicitly asked to change:

### Code Formatting
- **Line length**: 160 characters maximum (configured in rustfmt.toml)
- **Indentation**: 4 spaces (Rust standard)
- **Imports**: Automatically reordered with rustfmt

### Naming Conventions
- **Files/modules**: `snake_case` (e.g., `asset_id.rs`, `chain_address.rs`)
- **Crates**: Prefixed naming (`gem_*` for blockchains, `security_*` for security)
- **Functions/variables**: `snake_case` 
- **Structs/enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`

### Import Organization
1. Standard library imports first
2. External crate imports
3. Local crate imports  
4. Module re-exports with `pub use`

**IMPORTANT**: Always import models and types at the top of the file. Never use inline imports inside functions (e.g., `use crate::models::SomeType` inside a function). All imports must be declared in the file header section.

### Error Handling
- Use `thiserror` for custom error types
- Implement `From` traits for error conversion
- Consistent `Result<T, Error>` return types
- Propagate errors with `?` operator

### Database Patterns
- Separate database models from domain primitives
- Use `as_primitive()` methods for conversion
- Diesel ORM with PostgreSQL backend
- Transaction support and upsert operations

### Async Patterns
- Tokio runtime throughout
- Async client structs with `Result<T, Error>` methods
- `Arc<tokio::sync::Mutex<T>>` for shared async state

### Testing
- Integration tests in separate `tests/` directories
- `#[tokio::test]` for async tests
- Descriptive test names with `test_` prefix
- `Result<(), Box<dyn std::error::Error + Send + Sync>>` for test error handling
- Integration tests configured with `test = false` and `required-features` for manual execution
- Use real blockchain networks for RPC client testing (Ethereum mainnet, etc.)
- **Test Data Management**: For long JSON test data (>20 lines), prefer storing in `tests/data/` directory and loading with `include_str!()` macro rather than inline strings to improve readability and maintainability

## Repository Pattern

Services access repositories through direct methods on `DatabaseClient`. This pattern:
- Provides clean separation between data access and business logic
- Each repository handles a specific domain (assets, devices, etc.)
- DatabaseClient implements all repository traits directly
- Repository methods return primitive types, not database models
- Simplified API with direct method calls

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

Direct repository access methods available on DatabaseClient:
- `assets()` - Asset operations
- `devices()` - Device operations  
- `subscriptions()` - Subscription operations
- `prices()` - Price operations
- `transactions()` - Transaction operations
- And many more...

## Key Development Patterns

- Each blockchain has its own crate with unified RPC client patterns
- UniFFI bindings require careful Rust API design for mobile compatibility
- BigDecimal used throughout for financial precision
- Extensive use of async/await with Tokio
- Database models use Diesel ORM with automatic migrations
- Cross-platform compatibility considerations for mobile performance

### RPC Client Patterns
- Use `gem_jsonrpc::JsonRpcClient` for blockchain RPC interactions
- Prefer `alloy_primitives::hex::encode_prefixed()` for hex encoding with 0x prefix
- Use `alloy_primitives::Address::to_string()` instead of manual formatting
- RPC calls expect hex strings directly - avoid double encoding
- Batch operations use `JsonRpcClient::batch_call()` for multiple requests
- Error handling with `JsonRpcError` type and proper propagation

### Blockchain Provider Patterns
- Each blockchain crate has a `provider/` directory with trait implementations
- Provider methods should get raw data from RPC client, then use mapper functions for conversion
- Mapper functions are located in separate `*_mapper.rs` files for clean separation
- Example pattern: `get_balance_coin()` calls `self.get_balance()` then `balances_mapper::map_coin_balance()`
- This pattern ensures consistent data transformation and testability across all blockchain implementations

### Integration Testing
- Add integration tests for RPC functionality to verify real network compatibility
- Use `#[tokio::test]` for async integration tests
- Test with recent blocks for batch operations (more reliable than historical blocks)
- Integration tests should verify both successful calls and proper error handling
- Use realistic contract addresses (USDC, etc.) for eth_call testing

### Code Formatting and Commits
- **ALWAYS run `just format` before committing code**
- The formatter will automatically fix formatting issues across all crates
- This ensures consistent code style throughout the workspace
- Commit messages should be descriptive and follow conventional commit format