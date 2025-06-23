# Ethereum Tx Δ-Extractor (ETH & ERC-20)

## Goal
Given a transaction object or hash, fetch both native-ETH and ERC-20 token balance deltas via a single RPC call, and emit a list of `{ address, ethDelta, tokenDeltas: { [tokenAddr]: delta } }`.

## Architecture

The delta extractor is implemented in `crates/gem_evm/src/trace_call/` with the following structure:

```
gem_evm/src/trace_call/
├── mod.rs         # Main extraction logic + tests
├── model.rs       # All data models + model tests  
└── test_data.rs   # Real transaction test data
```

## 1. RPC Call

### JSON-RPC Implementation
The trace_call RPC method is implemented in multiple places:

**gem_evm/src/jsonrpc.rs:**
```rust
pub enum EthereumRpc {
    TraceCall(TransactionObject, Vec<String>, BlockParameter),
    // ... other methods
}
```

**gem_evm/src/rpc/client.rs:**
```rust
impl EthereumClient {
    /// Strongly typed trace_call that returns parsed TraceCallResult
    pub async fn trace_call(&self, tx_request: AlloyTransactionRequest, trace_types: Vec<String>) -> Result<TraceCallResult> {
        let params = (tx_request, trace_types, BlockId::Number(BlockNumberOrTag::Latest));
        Ok(self.client.request("trace_call", params).await?)
    }

    /// Raw trace_call that returns untyped JSON for advanced use cases
    pub async fn trace_call_raw(&self, tx_request: AlloyTransactionRequest, trace_types: Vec<String>) -> Result<Value> {
        let params = (tx_request, trace_types, BlockId::Number(BlockNumberOrTag::Latest));
        Ok(self.client.request("trace_call", params).await?)
    }
}
```

**gemstone/src/ethereum/jsonrpc.rs:**
```rust
/// Strongly typed trace_call that returns parsed TraceCallResult
pub async fn trace_call(provider: Arc<dyn AlienProvider>, chain: Chain, tx: TransactionObject, trace_types: Vec<String>) -> Result<TraceCallResult, SwapperError> {
    let call = EthereumRpc::TraceCall(tx, trace_types, BlockParameter::Latest);
    let client = JsonRpcClient::new_with_chain(provider, chain);
    let resp: JsonRpcResult<TraceCallResult> = client.call(&call).await?;
    Ok(resp.take()?)
}

/// Raw trace_call that returns untyped JSON for advanced use cases
pub async fn trace_call_raw(provider: Arc<dyn AlienProvider>, chain: Chain, tx: TransactionObject, trace_types: Vec<String>) -> Result<Value, SwapperError> {
    let call = EthereumRpc::TraceCall(tx, trace_types, BlockParameter::Latest);
    let client = JsonRpcClient::new_with_chain(provider, chain);
    let resp: JsonRpcResult<Value> = client.call(&call).await?;
    Ok(resp.take()?)
}
```

**Usage:**
```rust
const txObject = { from, to, data, value, gas, gasPrice }
const result = await provider.send("trace_call", [
  txObject,
  ["trace","stateDiff"],
  "latest"
])
// result.stateDiff → per-address ETH/storage diffs
// result.trace     → internal call frames
```

## 2. Data Models

**Core Models (gem_evm/src/trace_call/model.rs):**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddressDelta {
    pub address: Address,
    pub eth_delta: BigInt,
    pub token_deltas: HashMap<Address, BigInt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumDeltaResult {
    pub deltas: Vec<AddressDelta>,
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub address: Address,
    pub balance_slot: u32,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaExtractorConfig {
    pub tracked_tokens: Vec<TokenConfig>,
    pub include_trace: bool,
    pub include_state_diff: bool,
}
```

**Trace Response Models:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCallResult {
    #[serde(rename = "stateDiff")]
    pub state_diff: Option<HashMap<Address, StateDiff>>,
    pub trace: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDiff {
    #[serde(default)]
    pub balance: Option<HashMap<String, BalanceChange>>,
    #[serde(default)]
    pub storage: Option<HashMap<String, StorageChange>>,
    #[serde(default)]
    pub nonce: Option<HashMap<String, BalanceChange>>,
    #[serde(default)]
    pub code: Option<String>,
}
```

## 3. Parse Native-ETH Deltas

**Implementation (gem_evm/src/trace_call/mod.rs):**
```rust
// Extract native ETH deltas
for (addr, diff) in state_diff.iter() {
    if let Some(balance_changes) = &diff.balance {
        for (change_type, balance_change) in balance_changes {
            if change_type == "*" {
                if let (Ok(from_val), Ok(to_val)) = (
                    parse_hex_string_to_bigint(&balance_change.from),
                    parse_hex_string_to_bigint(&balance_change.to),
                ) {
                    let delta_eth = &to_val - &from_val;
                    if delta_eth != BigInt::from(0) {
                        deltas
                            .entry(addr.clone())
                            .or_default()
                            .eth = delta_eth;
                    }
                }
            }
        }
    }
}
```

## 4. Parse ERC-20 Token Deltas

**Implementation:**
```rust
// Extract ERC-20 token deltas
for (token_addr, slot_idx) in tracked_tokens {
    if let Some(token_diff) = state_diff.get(token_addr) {
        if let Some(storage_changes) = &token_diff.storage {
            for addr in participants {
                let slot_key = compute_mapping_slot_key(addr, *slot_idx);
                if let Some(storage_change) = storage_changes.get(&slot_key) {
                    match storage_change {
                        StorageChange::Change { change } => {
                            if let (Ok(from_val), Ok(to_val)) = (
                                parse_hex_string_to_bigint(&change.from),
                                parse_hex_string_to_bigint(&change.to),
                            ) {
                                let delta_tok = &to_val - &from_val;
                                if delta_tok != BigInt::from(0) {
                                    deltas
                                        .entry(addr.clone())
                                        .or_default()
                                        .tokens
                                        .insert(token_addr.clone(), delta_tok);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## 5. Storage Slot Key Computation

**Keccak256-based mapping key calculation:**
```rust
fn compute_mapping_slot_key(address: &str, slot_idx: u32) -> String {
    let addr_bytes = pad32_address(address);
    let slot_bytes = pad32_u32(slot_idx);
    let combined = [addr_bytes, slot_bytes].concat();
    let hash = keccak256(&combined);
    format!("0x{}", hex::encode(hash))
}

fn pad32_address(address: &str) -> Vec<u8> {
    let addr_str = address.strip_prefix("0x").unwrap_or(address);
    let mut addr_bytes = hex::decode(addr_str).expect("Invalid address hex");
    
    // Pad to 32 bytes
    let mut padded = vec![0u8; 32 - addr_bytes.len()];
    padded.append(&mut addr_bytes);
    padded
}
```

## 6. Main API Functions

**Core extraction function:**
```rust
pub fn extract_deltas(
    result: TraceCallResult,
    tracked_tokens: &HashMap<Address, u32>,
    participants: &[Address],
) -> HashMap<Address, Delta>
```

**High-level wrapper:**
```rust
pub fn extract_deltas_from_trace_result(
    result: TraceCallResult,
    config: &DeltaExtractorConfig,
    participants: &[Address],
) -> EthereumDeltaResult
```

**Address extraction from trace:**
```rust
pub fn extract_addresses_from_trace(trace: &Value) -> Vec<Address>
```

## 7. Configuration and Usage

**Setting up token tracking:**
```rust
let mut config = DeltaExtractorConfig::new();
config = config.add_token(
    TokenConfig::new("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string(), 9)
        .with_metadata("USDC".to_string(), 6)
);
config = config.add_token(
    TokenConfig::new("0xdac17f958d2ee523a2206206994597c13d831ec7".to_string(), 2)
        .with_metadata("USDT".to_string(), 6)
);
```

**Full extraction workflow with typed API:**
```rust
// 1. Make strongly typed trace_call RPC - no manual JSON parsing!
let trace_result: TraceCallResult = trace_call(provider, chain, tx_object, config.trace_types()).await?;

// 2. Direct access to typed fields
if let Some(state_diff) = &trace_result.state_diff {
    println!("Found state changes for {} addresses", state_diff.len());
}

// 3. Extract participant addresses from trace
let participants = if let Some(trace) = &trace_result.trace {
    extract_addresses_from_trace(trace)
} else {
    vec![] // fallback to manual participant list
};

// 4. Extract deltas using typed result
let result = extract_deltas_from_trace_result(trace_result, &config, &participants);

// 5. Process results with compile-time type safety
for delta in result.deltas {
    println!("Address {}: ETH Δ = {}", delta.address, delta.eth_delta);
    for (token_addr, token_delta) in delta.token_deltas {
        println!("  Token {} Δ = {}", token_addr, token_delta);
    }
}
```

**Benefits of typed API:**
- ✅ **Compile-time safety**: No runtime JSON parsing errors
- ✅ **IDE support**: Auto-completion and documentation
- ✅ **Self-documenting**: Clear return types show exactly what's available
- ✅ **Refactoring safety**: Changes to structs caught at compile time
- ✅ **Performance**: Validation happens once at deserialization time

## 8. Hex String Parsing

Uses the shared `serde_serializers::bigint::deserialize_bigint_from_str` function which supports both decimal and hex (0x-prefixed) strings:

```rust
fn parse_hex_string_to_bigint(hex_str: &str) -> Result<BigInt, String> {
    use serde::de::value::{Error as ValueError, StringDeserializer};
    use serde::de::IntoDeserializer;
    
    let deserializer: StringDeserializer<ValueError> = hex_str.to_string().into_deserializer();
    deserialize_bigint_from_str(deserializer).map_err(|e| e.to_string())
}
```

## 9. Testing

Comprehensive test suite with real transaction data from hash `0x825c8f677d215d4f128218aea1d9aa965d93790d8195f609ffb4fa6d4310fc79`:

- **Native ETH balance extraction**
- **ERC-20 token balance extraction (USDC & USDT)**
- **Storage slot key computation**
- **Address extraction from trace data**
- **Integration tests with full config setup**

Run tests: `cargo test trace_call::tests`

## 10. Known Token Slot Mappings

Common ERC-20 token balance slot indices:
- **USDC** (`0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48`): slot 9
- **USDT** (`0xdac17f958d2ee523a2206206994597c13d831ec7`): slot 2
- **WETH** (`0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2`): slot 3