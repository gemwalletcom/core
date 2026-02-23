# Tests

## Conventions

- Place integration tests in `tests/` directories
- Use `#[tokio::test]` for async tests
- Prefix test names descriptively with `test_`
- Use `Result<(), Box<dyn std::error::Error + Send + Sync>>` for test error handling
- Configure integration tests with `test = false` and appropriate `required-features` for manual execution
- Prefer real networks for RPC client tests (e.g., Ethereum mainnet)

## Use `.unwrap()`, not `.expect()` in Tests

```rust
// bad — unnecessary message in tests
let client = reqwest::Client::builder()
    .timeout(timeout)
    .build()
    .expect("failed to build reqwest client");

// good — concise, failure is obvious from test context
let client = reqwest::Client::builder()
    .timeout(timeout)
    .build()
    .unwrap();
```

## Test Data Management

For long JSON test data (>20 lines), store in `testdata/` and load with `include_str!()`. Per-crate layout is typically `src/`, `tests/`, `testdata/`.

```rust
// good — external test data
let response: ApiResponse = serde_json::from_str(
    include_str!("../../testdata/balances_response.json")
).unwrap();
```

## Mock Pattern with `testkit/` Modules

Add `mock()` constructors in `testkit/` modules instead of building structs inline in tests:

```rust
// good — crates/gem_hypercore/src/testkit.rs
impl AssetPositions {
    pub fn mock() -> Self {
        Self {
            asset_positions: vec![],
            margin_summary: MarginSummary {
                account_value: "10000".to_string(),
                total_ntl_pos: "5000".to_string(),
                total_raw_usd: "5000".to_string(),
                total_margin_used: "2000".to_string(),
            },
            // ...
        }
    }
}

// good — parameterized mock
impl OpenOrder {
    pub fn mock(coin: &str, oid: u64, order_type: &str, trigger_px: f64, limit_px: Option<f64>) -> Self {
        Self {
            coin: coin.to_string(),
            oid,
            trigger_px: Some(trigger_px),
            limit_px,
            is_position_tpsl: true,
            order_type: order_type.to_string(),
        }
    }
}
```

## Direct `assert_eq!`

Derive `PartialEq` on test-relevant enums and use `assert_eq!` with constructed expected values:

```rust
// bad — destructuring with panic
let result = parse_action(input);
let Action::SignMessage { chain, data, .. } = result else {
    panic!("Expected SignMessage");
};
assert_eq!(chain, Chain::Ethereum);

// good — direct comparison
assert_eq!(result, Action::SignMessage {
    chain: Chain::Ethereum,
    sign_type: SignDigestType::Eip191,
    data: "hello".to_string(),
});
```

## Test Helpers

Create concise constructor functions for frequently constructed enum variants in test modules:

```rust
// good — helper avoids repetitive boilerplate
fn sign_message(chain: Chain, sign_type: SignDigestType, data: &str) -> WalletConnectAction {
    WalletConnectAction::SignMessage {
        chain,
        sign_type,
        data: data.to_string(),
    }
}
```

## Integration Testing

- Add integration tests for RPC functionality to verify real network compatibility
- Prefer recent blocks for batch operations (more reliable than historical blocks)
- Verify both successful calls and proper error propagation
- Use realistic contract addresses (e.g., USDC) for `eth_call` testing
