# Defensive Programming

Safety rules to prevent bugs and maintain production reliability.

## No `matches!` — Use Exhaustive `match`

The `matches!` macro silently ignores new variants added later. Use exhaustive `match` with explicit arms.

```rust
// bad — new variants silently return false
fn is_transfer(action: &Action) -> bool {
    matches!(action, Action::Transfer { .. })
}

// good — compiler forces handling all variants
fn is_transfer(action: &Action) -> bool {
    match action {
        Action::Transfer { .. } => true,
        Action::Swap { .. } | Action::Stake { .. } | Action::Sign { .. } => false,
    }
}
```

## No `#[allow(dead_code)]`

Remove dead code instead of suppressing warnings. Dead code increases maintenance burden and hides actual issues.

```rust
// bad
#[allow(dead_code)]
fn old_calculation(x: u64) -> u64 { x * 2 }

// good — delete it entirely
```

## No `todo!()` / `unimplemented!()`

Implement the functionality or return an error. Panicking macros in production code cause crashes.

```rust
// bad — panics at runtime
fn get_fee(chain: Chain) -> u64 {
    todo!("implement fee calculation")
}

// good — return error for unhandled cases
fn get_fee(chain: Chain) -> Result<u64, Error> {
    match chain {
        Chain::Ethereum => Ok(21000),
        Chain::Bitcoin => Ok(1000),
        _ => Err(Error::UnsupportedChain(chain)),
    }
}
```

## No `println!` in Production Code

Use structured logging (`tracing` crate) instead of `println!` in service code. `println!` bypasses log levels, timestamps, and monitoring integrations.

```rust
// bad — apps/daemon/src/worker/prices/charts_updater.rs
println!("update charts {}", coin_id.id.clone());
println!("update charts error: {err}");

// bad — apps/api/src/main.rs
println!("api start service: {}", service.as_ref());

// good — structured logging with context
tracing::info!(coin_id = %coin_id.id, "updating charts");
tracing::error!(%err, "charts update failed");
tracing::info!(service = %service.as_ref(), "api service starting");
```

## No `.unwrap()` / `.expect()` in Production Code

Return `Result` instead. Panicking in production causes service crashes.

```rust
// bad — panics on None/Err
let value = map.get("key").unwrap();
let data = serde_json::from_str(input).expect("invalid json");

// good — propagate errors
let value = map.get("key").ok_or(Error::MissingKey("key"))?;
let data: MyStruct = serde_json::from_str(input)?;
```

Note: `.unwrap()` is fine in tests — see [Tests](tests.md).

## Prefer Immutable Bindings

Use `mut` only when truly necessary. Immutable bindings prevent accidental mutation.

```rust
// bad — unnecessary mut
let mut result = calculate_fee(amount);
return result;

// good
let result = calculate_fee(amount);
return result;
```
