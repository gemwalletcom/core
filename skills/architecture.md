# Architecture

## Key Principles

- One crate per blockchain using unified RPC client patterns
- UniFFI bindings require careful Rust API design for mobile compatibility
- Use `BigDecimal` for financial precision
- Use async/await with Tokio across services
- Database models use Diesel ORM with automatic migrations
- Consider cross-platform performance constraints for mobile

## Provider/Mapper Pattern

Each blockchain crate has a `provider/` directory with trait implementations. Provider methods fetch raw data via RPC, then call mapper functions for conversion. Place mapper functions in separate `*_mapper.rs` files.

```rust
// good — provider delegates to mapper (crates/gem_hypercore/src/provider/balances.rs)
use super::balances_mapper::{map_balance_coin, map_balance_staking, map_balance_tokens};

#[async_trait]
impl<C: Client> ChainBalance for HyperCoreClient<C> {
    async fn get_balance_coin(&self, address: &str) -> Result<CoinBalance, Box<dyn Error + Send + Sync>> {
        let available = self.get_balance(address).await?;
        Ok(map_balance_coin(available, self.chain))
    }
}
```

```rust
// good — provider/staking.rs calls staking_mapper
async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Send + Sync>> {
    let validators = self.get_validators().await?;
    Ok(staking_mapper::map_staking_validators(validators, self.chain, apy))
}
```

This pattern ensures consistent data transformation and testability across all blockchain implementations.

## Repository Pattern

Services access repositories through direct methods on `DatabaseClient`:

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

Available repository accessors on `DatabaseClient`:
- `assets()` — Asset operations
- `devices()` — Device operations
- `subscriptions()` — Subscription operations
- `prices()` — Price operations
- `transactions()` — Transaction operations
- And more...

Key properties:
- Separates data access and business logic
- Assigns each repository a specific domain
- Implements all repository traits directly on `DatabaseClient`
- Returns primitive types from repository methods, not database models

## RPC Client Patterns

- Use `gem_jsonrpc::JsonRpcClient` for blockchain RPC interactions
- **Use `primitives::hex`** for hex encoding/decoding (not `alloy_primitives::hex`)
- RPC calls expect hex strings directly; avoid double encoding
- Use `JsonRpcClient::batch_call()` for batch operations
- Propagate errors via `JsonRpcError`

## UniFFI Patterns

Use `#[uniffi::remote]` for wrapper types around external models instead of creating duplicate structs with `From` implementations:

```rust
// good — uniffi::remote avoids duplication
use primitives::AuthNonce;
pub type GemAuthNonce = AuthNonce;
#[uniffi::remote(Record)]
pub struct GemAuthNonce { pub nonce: String, pub timestamp: u32 }
```

## Shared Utilities

- **U256 conversions**: Prefer `u256_to_biguint` and `biguint_to_u256` from `crates/gem_evm/src/u256.rs` for Alloy `U256` <-> `BigUint` conversions
