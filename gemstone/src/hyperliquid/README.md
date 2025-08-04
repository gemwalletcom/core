# Hyperliquid Integration

This module provides Rust bindings for interacting with the Hyperliquid DEX platform, including action creation, signing, and EIP-712 typed data generation.

## Overview

Hyperliquid is a high-performance decentralized perpetual futures exchange. This integration supports:

- **Trading Actions**: Market orders, limit orders, position management
- **Account Management**: Leverage updates, referrer settings, agent approvals
- **Withdrawal Operations**: Asset withdrawals to external addresses
- **EIP-712 Signing**: Typed data generation for MetaMask and hardware wallet compatibility

## Architecture

### Module Structure

```
hyperliquid/
├── actions/           # Action data structures
│   ├── mod.rs        # HyperCoreModelFactory and exports
│   ├── order.rs      # Trading orders (market, limit)
│   ├── update_leverage.rs  # Leverage management
│   ├── set_referrer.rs     # Referrer code setting
│   ├── withdrawal.rs       # Asset withdrawals
│   ├── approve_agent.rs    # Agent approvals
│   └── approve_builder_fee.rs  # Builder fee approvals
├── core/             # Core signing and hashing logic
│   ├── hypercore.rs  # EIP-712 typed data generation
│   ├── eip712.rs     # EIP-712 utilities
│   ├── hasher.rs     # Action hashing for signatures
│   └── models.rs     # Internal models (PhantomAgent)
└── test/             # Test data and fixtures
```

## Key Concepts

### 1. Action Types

There are two main categories of actions in Hyperliquid:

#### L1 Actions (Agent-Signed)
Actions that are signed by an agent key and submitted on behalf of a user:
- `order` - Place market/limit orders
- `updateLeverage` - Change leverage settings
- `setReferrer` - Set referral code

#### User-Signed Actions
Actions that must be signed directly by the user's private key:
- `withdraw` - Withdraw assets to external addresses  
- `approveAgent` - Approve an agent to trade on behalf of user
- `approveBuilderFee` - Approve builder fee payments

### 2. Field Order Importance

**CRITICAL**: Field order in structs matters for msgpack serialization and hash calculation.

```rust
// CORRECT order - matches Python SDK
#[derive(serde::Serialize)]
pub struct HyperUpdateLeverage {
    pub r#type: String,        // 1st: "updateLeverage"
    pub asset: u32,            // 2nd: asset index
    pub is_cross: bool,        // 3rd: cross/isolated margin
    pub leverage: u64,         // 4th: leverage value
}
```

**Do NOT change field order** unless you verify the exact order from the Python SDK. Incorrect order will result in invalid signatures and failed transactions.

### 3. Action Hash Generation

Actions are hashed using a specific algorithm for signature generation:

```rust
let action_value = serde_json::to_value(&action).unwrap();
let hash = action_hash(&action_value, None, nonce, None).unwrap();
```

The hash is used to create a `PhantomAgent` which generates the connection ID for EIP-712 signing.

### 4. EIP-712 Typed Data

#### L1 Actions (Agent Signing)
```rust
let hypercore = HyperCore::new();
let typed_data = hypercore.update_leverage_typed_data(update_leverage, nonce);
```

#### User Actions (User Signing)  
```rust
let typed_data = hypercore.withdrawal_request_typed_data(withdrawal);
```

The typed data can be used with MetaMask's `eth_signTypedData_v4` or hardware wallets.

## Usage Examples

### Creating a Market Order

```rust
use crate::hyperliquid::actions::*;

let factory = HyperCoreModelFactory::new();

// Buy 0.28 units of asset 5 at market price
let order = factory.make_market_order(
    5,                           // asset index
    true,                        // is_buy (true = long, false = short)
    "200.21".to_string(),        // price
    "0.28".to_string(),          // size
    false,                       // reduce_only
    None                         // builder (optional)
);

// Serialize for API submission
let action_json = factory.serialize_order(&order);
```

### Updating Leverage

```rust
// Set 10x cross leverage for asset 25
let update_leverage = factory.make_update_leverage(
    25,    // asset index
    true,  // is_cross (true = cross margin, false = isolated)
    10     // leverage multiplier
);

// Generate EIP-712 typed data for signing
let hypercore = HyperCore::new();
let nonce = get_timestamp_ms();
let typed_data = hypercore.update_leverage_typed_data(update_leverage, nonce);
```

### Setting Referrer Code

```rust
let set_referrer = factory.make_set_referrer("GEMWALLET".to_string());
let action_json = factory.serialize_set_referrer(&set_referrer);

// For L1 signing
let typed_data = hypercore.set_referrer_typed_data(set_referrer, nonce);
```

### Withdrawing Assets

```rust
// Withdraw 2 USDC to external address
let withdrawal = factory.make_withdraw(
    "2".to_string(),           // amount
    "0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7".to_string(), // destination
    nonce                      // nonce timestamp
);

// This requires USER signing (not agent)
let typed_data = hypercore.withdrawal_request_typed_data(withdrawal);
```

## Signing Workflow

### 1. L1 Actions (Agent-Signed)

```rust
// 1. Create action
let action = factory.make_update_leverage(asset, is_cross, leverage);

// 2. Generate typed data
let typed_data = hypercore.update_leverage_typed_data(action, nonce);

// 3. Sign with agent private key
let signature = sign_typed_data(agent_private_key, &typed_data);

// 4. Build final request
let signed_request = factory.build_signed_request(signature, action_json, nonce);

// 5. Submit to Hyperliquid API
post_to_exchange(signed_request);
```

### 2. User Actions (User-Signed)

```rust
// 1. Create action  
let withdrawal = factory.make_withdraw(amount, address, nonce);

// 2. Generate typed data
let typed_data = hypercore.withdrawal_request_typed_data(withdrawal);

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

The module includes comprehensive tests that verify:

- Action serialization matches expected JSON format
- EIP-712 typed data generation 
- Field ordering for msgpack compatibility
- Address lowercasing for consistency
- Signature verification against test vectors

Run tests:
```bash
cargo test -p gemstone --lib hyperliquid
```

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
- **NEVER** change field order in action structs
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

- [Hyperliquid Documentation](https://hyperliquid.gitbook.io/hyperliquid-docs/)
- [Python SDK](https://github.com/hyperliquid-dex/hyperliquid-python-sdk)
- [API Endpoints](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/exchange-endpoint)
- [EIP-712 Specification](https://eips.ethereum.org/EIPS/eip-712)