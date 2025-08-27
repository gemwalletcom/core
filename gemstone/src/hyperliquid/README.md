# Hyperliquid Integration

This module provides Rust bindings for interacting with the Hyperliquid DEX platform, including action creation, signing, and EIP-712 typed data generation.

## Overview

Hyperliquid is a high-performance decentralized perpetual futures exchange. This integration supports:

- **Trading Actions**: Market orders, limit orders, position management
- **Account Management**: Leverage updates, referrer settings, agent approvals
- **Withdrawal Operations**: Asset withdrawals to external addresses
- **EIP-712 Signing**: Typed data generation for MetaMask and hardware wallet compatibility

## Architecture

This module serves as a UniFFI binding layer for mobile platforms (iOS/Android), providing access to Hyperliquid functionality implemented in the `gem_hypercore` crate.

### Module Structure

```
hyperliquid/
├── remote_models.rs   # UniFFI remote model declarations
├── mod.rs            # Module exports and mobile interface
└── README.md         # This documentation
```

### Core Implementation

The actual Hyperliquid implementation resides in `gem_hypercore` crate:

```
gem_hypercore/
├── src/
│   ├── actions/           # Action data structures
│   │   ├── agent_sign/    # L1 payload actions (order, set_referrer, update_leverage)
│   │   └── user_sign/     # User-signed actions (withdrawal, approve_agent, etc.)
│   ├── core/             # Core signing and hashing logic
│   │   ├── hypercore.rs  # EIP-712 typed data generation functions
│   │   ├── eip712.rs     # EIP-712 utilities
│   │   ├── hasher.rs     # Action hashing for signatures
│   │   └── models.rs     # Internal models and types
│   └── provider/         # Data providers for balances, positions, etc.
└── tests/
    └── data/             # Test fixtures and real API response data
```

## Key Concepts

### 1. Action Types

There are two main categories of actions in Hyperliquid, now organized in `gem_hypercore`:

#### Agent Sign Actions (`gem_hypercore::actions::agent_sign`)
Actions that require L1 payload signing:
- `PlaceOrder` - Market/limit orders with optional TP/SL
- `UpdateLeverage` - Change leverage settings
- `SetReferrer` - Set referral code

#### User Sign Actions (`gem_hypercore::actions::user_sign`)
Actions that must be signed directly by the user's private key:
- `WithdrawalRequest` - Withdraw assets to external addresses  
- `ApproveAgent` - Approve an agent to trade on behalf of user
- `ApproveBuilderFee` - Approve builder fee payments
- `SpotSend` - Transfer spot tokens
- `UsdSend` - Send USD to addresses
- `UsdClassTransfer` - Transfer between spot and perps
- `CDeposit` - Deposit to staking
- `TokenDelegate` - Delegate/undelegate tokens
- `Cancel` - Cancel existing orders

### 2. Field Order Importance

**CRITICAL**: Field order in structs matters for msgpack serialization and hash calculation.

```rust
// CORRECT order - matches Python SDK (in gem_hypercore)
#[derive(serde::Serialize)]
pub struct UpdateLeverage {
    pub r#type: String,        // 1st: "updateLeverage"
    pub asset: u32,            // 2nd: asset index
    pub is_cross: bool,        // 3rd: cross/isolated margin
    pub leverage: u64,         // 4th: leverage value
}
```

**Do NOT change field order** in `gem_hypercore` structs unless you verify the exact order from the Python SDK. Incorrect order will result in invalid signatures and failed transactions.

### 3. Action Hash Generation

Actions are hashed using a specific algorithm for signature generation:

```rust
let action_value = serde_json::to_value(&action).unwrap();
let hash = action_hash(&action_value, None, nonce, None).unwrap();
```

The hash is used to create a `PhantomAgent` which generates the connection ID for EIP-712 signing.

### 4. EIP-712 Typed Data

EIP-712 typed data generation is now handled by standalone functions in `gem_hypercore::core::hypercore`:

#### Agent Sign Actions
```rust
use gem_hypercore::core::hypercore;

let typed_data = hypercore::update_leverage_typed_data(update_leverage, nonce);
let typed_data = hypercore::place_order_typed_data(order, nonce);
let typed_data = hypercore::set_referrer_typed_data(referrer, nonce);
```

#### User Sign Actions  
```rust
let typed_data = hypercore::withdrawal_request_typed_data(withdrawal);
let typed_data = hypercore::approve_agent_typed_data(agent);
let typed_data = hypercore::approve_builder_fee_typed_data(fee);
// ... and other user sign functions
```

The typed data can be used with MetaMask's `eth_signTypedData_v4` or hardware wallets.

### 5. UniFFI Remote Models

This module provides UniFFI bindings through remote model declarations:

```rust
// Type aliases for external API (maintain "Hyper" prefix for backward compatibility)
pub type HyperPlaceOrder = actions::PlaceOrder;
pub type HyperUpdateLeverage = actions::UpdateLeverage;

// Remote declarations for UniFFI
#[uniffi::remote(Record)]
pub struct HyperPlaceOrder { /* ... */ }

// Exported functions for mobile platforms
#[uniffi::export]
pub fn hyper_core_place_order_typed_data(order: HyperPlaceOrder, nonce: u64) -> String {
    hypercore::place_order_typed_data(order, nonce)
}
```

## Usage Examples

All core functionality is now accessed through UniFFI exported functions that delegate to `gem_hypercore`:

### Creating a Market Order

```rust
// Use UniFFI exported functions from this module
let order = hyper_make_market_order(
    5,                           // asset index
    true,                        // is_buy (true = long, false = short)
    "200.21".to_string(),        // price
    "0.28".to_string(),          // size
    false,                       // reduce_only
    None                         // builder (optional)
);

// Serialize for API submission
let action_json = hyper_serialize_order(&order);

// Generate EIP-712 typed data for signing
let nonce = get_timestamp_ms();
let typed_data = hyper_core_place_order_typed_data(order, nonce);
```

### Updating Leverage

```rust
// Set 10x cross leverage for asset 25
let update_leverage = hyper_make_update_leverage(
    25,    // asset index
    true,  // is_cross (true = cross margin, false = isolated)
    10     // leverage multiplier
);

// Generate EIP-712 typed data for signing
let typed_data = hyper_core_update_leverage_typed_data(update_leverage, nonce);
```

### Setting Referrer Code

```rust
let set_referrer = hyper_make_set_referrer("GEMWALLET".to_string());
let action_json = hyper_serialize_set_referrer(&set_referrer);

// For L1 signing
let typed_data = hyper_core_set_referrer_typed_data(set_referrer, nonce);
```

### Withdrawing Assets

```rust
// Withdraw 2 USDC to external address
let withdrawal = hyper_make_withdraw(
    "2".to_string(),           // amount
    "0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7".to_string(), // destination
    nonce                      // nonce timestamp
);

// This requires USER signing (not agent)
let typed_data = hyper_core_withdrawal_request_typed_data(withdrawal);
```

## Signing Workflow

### 1. Agent Sign Actions

```rust
// 1. Create action using UniFFI exports
let action = hyper_make_update_leverage(asset, is_cross, leverage);

// 2. Generate typed data
let typed_data = hyper_core_update_leverage_typed_data(action, nonce);

// 3. Sign with agent private key
let signature = sign_typed_data(agent_private_key, &typed_data);

// 4. Build final request
let signed_request = hyper_build_signed_request(signature, action_json, nonce);

// 5. Submit to Hyperliquid API
post_to_exchange(signed_request);
```

### 2. User Sign Actions

```rust
// 1. Create action using UniFFI exports
let withdrawal = hyper_make_withdraw(amount, address, nonce);

// 2. Generate typed data
let typed_data = hyper_core_withdrawal_request_typed_data(withdrawal);

// 3. Sign with USER private key (not agent)
let signature = sign_typed_data(user_private_key, &typed_data);

// 4. Submit to Hyperliquid API
post_to_exchange_with_user_signature(withdrawal, signature);
```

## API Integration

### Request Format

All requests to Hyperliquid follow this structure:

```json
{
  "action": { ... },           // The action object
  "signature": {
    "r": "0x...",             // Signature R component  
    "s": "0x...",             // Signature S component
    "v": 27                   // Recovery ID
  },
  "nonce": 1677777606040,     // Timestamp in milliseconds
  "isFrontend": true
}
```

### Headers

```
Content-Type: application/json
```

### Endpoints

- **Mainnet**: `https://api.hyperliquid.xyz/exchange`
- **Testnet**: `https://api.hyperliquid-testnet.xyz/exchange`

## Testing

Comprehensive tests are located in the `gem_hypercore` crate and verify:

- Action serialization matches expected JSON format
- EIP-712 typed data generation 
- Field ordering for msgpack compatibility
- Address lowercasing for consistency
- Signature verification against test vectors

Run tests:
```bash
# Test the core implementation
cargo test --package gem_hypercore

# Test UniFFI bindings
cargo test -p gemstone --lib hyperliquid
```

Test data is stored in `gem_hypercore/tests/data/` with real Hyperliquid API responses.

## Security Considerations

### Private Key Management
- **Agent Keys**: Used for L1 actions, can be rotated
- **User Keys**: Required for withdrawals and approvals, never share

### Address Handling
- All addresses are automatically lowercased for consistency
- Use checksummed addresses for display, lowercase for signing

### Nonce Management
- Use current timestamp in milliseconds as nonce
- Ensure nonces are always increasing to prevent replay attacks

### Field Order
- **NEVER** change field order in `gem_hypercore` action structs
- Verify against Python SDK when adding new actions
- Include warning comments about field order importance

## Error Handling

Common error scenarios:

1. **Invalid Field Order**: Results in signature mismatch
2. **Wrong Signature Type**: L1 vs User signing confusion  
3. **Address Case Mismatch**: Use lowercase addresses
4. **Stale Nonce**: Ensure nonce is recent timestamp
5. **Invalid Asset Index**: Verify asset exists on platform

## References

- [gem_hypercore README](../../../crates/gem_hypercore/README.md) - Core implementation documentation
- [Hyperliquid Documentation](https://hyperliquid.gitbook.io/hyperliquid-docs/)
- [Python SDK](https://github.com/hyperliquid-dex/hyperliquid-python-sdk)
- [API Endpoints](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [EIP-712 Specification](https://eips.ethereum.org/EIPS/eip-712)
- [UniFFI Documentation](https://mozilla.github.io/uniffi-rs/) - For mobile bindings