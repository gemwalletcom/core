# Common Issues

Known anti-patterns found in the codebase and their fixes.

## `alloy_primitives::hex` — Use `primitives::hex`

Several files import `alloy_primitives::hex` directly. Always use `primitives::hex` for consistency.

```rust
// bad — direct alloy import
use alloy_primitives::hex;
let bytes = hex::decode(input)?;

// good — use the project's re-export
use primitives::hex;
let bytes = hex::decode(input)?;
```

Known occurrences:
- `crates/gem_hypercore/src/signer/core_signer.rs`
- `crates/gem_hypercore/src/core/hahser.rs`
- `crates/signer/src/eip712/hash_impl.rs`
- `crates/gem_rewards/src/transfer_provider/evm/provider.rs`

## Misspelled File: `hahser.rs`

`crates/gem_hypercore/src/core/hahser.rs` should be `hasher.rs`. Fix when touching this file.

## Duplicate Constants

Before defining a new constant, check `crates/primitives/src/asset_constants.rs` for existing definitions. Reuse rather than redefine.

## Inline `use` in Diesel Query Functions

Diesel DSL imports (e.g., `use crate::schema::assets::dsl::*`) inside query functions are the **one exception** to the no-inline-imports rule. This is idiomatic Diesel usage and prevents DSL name collisions at module scope.

```rust
// acceptable — Diesel DSL exception
fn get_asset(conn: &mut PgConnection, id: &str) -> QueryResult<AssetRow> {
    use crate::schema::assets::dsl::*;
    assets.filter(asset_id.eq(id)).first(conn)
}
```

## `println!` in Service Code

Found in `apps/api/` and `apps/daemon/`. Replace with `tracing::info!`/`tracing::error!` — see [Defensive Programming](defensive-programming.md#no-println-in-production-code).

## Technical Debt Markers

The codebase has ~20 `TODO`/`FIXME` comments marking deferred work. Key areas:
- **Deprecated API endpoints** (`apps/api/src`): Old `/notifications`, `/wallets`, `/price_alerts` routes pending removal after client migration
- **Hardcoded fees**: Stellar transaction fee is hardcoded as `"1000"` string
- **Gas estimation**: Thorchain memo byte-length gas limits marked FIXME
- **Swap status**: Thorchain refunded transactions default to `Failed` status

When working near these areas, consider resolving the TODO if scope permits.
