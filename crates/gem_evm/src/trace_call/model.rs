use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use num_bigint::BigInt;

pub type Address = String;
pub type SlotKey = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddressDelta {
    pub address: Address,
    pub eth_delta: BigInt,
    pub token_deltas: HashMap<Address, BigInt>,
}

impl AddressDelta {
    pub fn new(address: Address) -> Self {
        Self {
            address,
            eth_delta: BigInt::from(0),
            token_deltas: HashMap::new(),
        }
    }

    pub fn has_deltas(&self) -> bool {
        self.eth_delta != BigInt::from(0) || !self.token_deltas.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumDeltaResult {
    pub deltas: Vec<AddressDelta>,
    pub transaction_hash: Option<String>,
}

impl EthereumDeltaResult {
    pub fn new() -> Self {
        Self {
            deltas: Vec::new(),
            transaction_hash: None,
        }
    }

    pub fn with_transaction_hash(mut self, hash: String) -> Self {
        self.transaction_hash = Some(hash);
        self
    }

    pub fn add_delta(&mut self, delta: AddressDelta) {
        if delta.has_deltas() {
            self.deltas.push(delta);
        }
    }

    pub fn get_addresses_with_deltas(&self) -> Vec<&Address> {
        self.deltas.iter().map(|d| &d.address).collect()
    }
}

impl Default for EthereumDeltaResult {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub address: Address,
    pub balance_slot: u32,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

impl TokenConfig {
    pub fn new(address: Address, balance_slot: u32) -> Self {
        Self {
            address,
            balance_slot,
            symbol: None,
            decimals: None,
        }
    }

    pub fn with_metadata(mut self, symbol: String, decimals: u8) -> Self {
        self.symbol = Some(symbol);
        self.decimals = Some(decimals);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaExtractorConfig {
    pub tracked_tokens: Vec<TokenConfig>,
    pub include_trace: bool,
    pub include_state_diff: bool,
}

impl Default for DeltaExtractorConfig {
    fn default() -> Self {
        Self {
            tracked_tokens: Vec::new(),
            include_trace: true,
            include_state_diff: true,
        }
    }
}

impl DeltaExtractorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tokens(mut self, tokens: Vec<TokenConfig>) -> Self {
        self.tracked_tokens = tokens;
        self
    }

    pub fn add_token(mut self, token: TokenConfig) -> Self {
        self.tracked_tokens.push(token);
        self
    }

    pub fn trace_types(&self) -> Vec<String> {
        let mut types = Vec::new();
        if self.include_trace {
            types.push("trace".to_string());
        }
        if self.include_state_diff {
            types.push("stateDiff".to_string());
        }
        types
    }

    pub fn get_token_slot_mapping(&self) -> HashMap<Address, u32> {
        self.tracked_tokens
            .iter()
            .map(|t| (t.address.clone(), t.balance_slot))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub eth: BigInt,
    pub tokens: HashMap<Address, BigInt>,
}

impl Delta {
    pub fn new() -> Self {
        Delta {
            eth: BigInt::from(0),
            tokens: HashMap::new(),
        }
    }
}

impl Default for Delta {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StorageChange {
    Change {
        #[serde(rename = "*")]
        change: BalanceChange,
    },
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceCallResult {
    #[serde(rename = "stateDiff")]
    pub state_diff: Option<HashMap<Address, StateDiff>>,
    pub trace: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_delta_creation() {
        let addr = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string();
        let delta = AddressDelta::new(addr.clone());
        
        assert_eq!(delta.address, addr);
        assert_eq!(delta.eth_delta, BigInt::from(0));
        assert!(delta.token_deltas.is_empty());
        assert!(!delta.has_deltas());
    }

    #[test]
    fn test_address_delta_with_changes() {
        let addr = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string();
        let mut delta = AddressDelta::new(addr.clone());
        
        delta.eth_delta = BigInt::from(1000);
        assert!(delta.has_deltas());
        
        delta.eth_delta = BigInt::from(0);
        delta.token_deltas.insert("0xtoken".to_string(), BigInt::from(500));
        assert!(delta.has_deltas());
    }

    #[test]
    fn test_config_trace_types() {
        let config = DeltaExtractorConfig::new();
        let types = config.trace_types();
        
        assert!(types.contains(&"trace".to_string()));
        assert!(types.contains(&"stateDiff".to_string()));
    }

    #[test]
    fn test_token_config() {
        let token = TokenConfig::new("0xtoken".to_string(), 0)
            .with_metadata("USDC".to_string(), 6);
        
        assert_eq!(token.address, "0xtoken");
        assert_eq!(token.balance_slot, 0);
        assert_eq!(token.symbol, Some("USDC".to_string()));
        assert_eq!(token.decimals, Some(6));
    }
}