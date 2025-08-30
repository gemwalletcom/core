use crate::block_explorer::BlockExplorer;
use std::collections::HashMap;

pub const TX_PATH: &str = "/tx";
pub const TXN_PATH: &str = "/txn";
pub const TXNS_PATH: &str = "/txns";
pub const TRANSACTION_PATH: &str = "/transaction";
pub const TRANSACTIONS_PATH: &str = "/transactions";
pub const ADDRESS_PATH: &str = "/address";
pub const ACCOUNT_PATH: &str = "/account";
pub const TOKEN_PATH: &str = "/token";
pub const COIN_PATH: &str = "/coin";
pub const VALIDATOR_PATH: &str = "/validator";
pub const VALIDATORS_PATH: &str = "/validators";
pub const ASSETS_PATH: &str = "/assets";
pub const ASSET_PATH: &str = "/asset";

#[derive(Debug, Clone)]
pub struct Metadata {
    pub name: &'static str,
    pub base_url: &'static str,
    pub tx_path: &'static str,
    pub address_path: &'static str,
    pub token_path: Option<&'static str>,
    pub validator_path: Option<&'static str>,
}

impl Metadata {
    /// Create a common explorer with /tx and /address paths (most common pattern)
    pub fn new(name: &'static str, base_url: &'static str) -> Self {
        Self {
            name,
            base_url,
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        }
    }

    /// Create a common explorer with /tx, /address, and /token paths
    pub fn with_token(name: &'static str, base_url: &'static str) -> Self {
        Self {
            name,
            base_url,
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: None,
        }
    }

    /// Create a validator-enabled explorer with /tx, /address, and /validator paths
    pub fn with_validator(name: &'static str, base_url: &'static str) -> Self {
        Self {
            name,
            base_url,
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: Some(VALIDATOR_PATH),
        }
    }

    /// Create a full-featured explorer with all standard paths
    pub fn full(name: &'static str, base_url: &'static str) -> Self {
        Self {
            name,
            base_url,
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: Some(VALIDATOR_PATH),
        }
    }

    /// Create an explorer using /transaction path instead of /tx (Blockchair style)
    pub fn blockchair(name: &'static str, base_url: &'static str) -> Self {
        Self {
            name,
            base_url,
            tx_path: TRANSACTION_PATH,
            address_path: ADDRESS_PATH,
            token_path: None,
            validator_path: None,
        }
    }

    /// Create a Mintscan-style explorer with assets and validators
    pub fn mintscan(name: &'static str, base_url: &'static str) -> Self {
        Self {
            name,
            base_url,
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(ASSETS_PATH),
            validator_path: Some(VALIDATORS_PATH),
        }
    }
}

pub struct Explorer {
    config: Metadata,
}

impl Explorer {
    pub fn boxed(config: Metadata) -> Box<dyn BlockExplorer> {
        Box::new(Self { config }) as Box<dyn BlockExplorer>
    }
}

impl BlockExplorer for Explorer {
    fn name(&self) -> String {
        self.config.name.into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}{}/{}", self.config.base_url, self.config.tx_path, hash)
    }

    fn get_address_url(&self, address: &str) -> String {
        format!("{}{}/{}", self.config.base_url, self.config.address_path, address)
    }

    fn get_token_url(&self, token: &str) -> Option<String> {
        self.config.token_path.map(|path| format!("{}{}/{}", self.config.base_url, path, token))
    }

    fn get_validator_url(&self, validator: &str) -> Option<String> {
        self.config.validator_path.map(|path| format!("{}{}/{}", self.config.base_url, path, validator))
    }
}

pub struct MultiChainExplorer {
    configs: HashMap<&'static str, Metadata>,
}

impl Default for MultiChainExplorer {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiChainExplorer {
    pub fn new() -> Self {
        Self { configs: HashMap::new() }
    }

    pub fn add_chain(mut self, chain: &'static str, config: Metadata) -> Self {
        self.configs.insert(chain, config);
        self
    }

    pub fn for_chain(&self, chain: &'static str) -> Option<Box<dyn BlockExplorer>> {
        self.configs
            .get(chain)
            .map(|config| Box::new(Explorer { config: config.clone() }) as Box<dyn BlockExplorer>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_explorer_basic_urls() {
        let config = Metadata {
            name: "TestExplorer",
            base_url: "https://test.com",
            tx_path: TX_PATH,
            address_path: ADDRESS_PATH,
            token_path: Some(TOKEN_PATH),
            validator_path: Some(VALIDATOR_PATH),
        };
        let explorer = Explorer::boxed(config);

        assert_eq!(explorer.name(), "TestExplorer");
        assert_eq!(explorer.get_tx_url("abc123"), "https://test.com/tx/abc123");
        assert_eq!(explorer.get_address_url("addr123"), "https://test.com/address/addr123");
        assert_eq!(explorer.get_token_url("token123"), Some("https://test.com/token/token123".to_string()));
        assert_eq!(explorer.get_validator_url("val123"), Some("https://test.com/validator/val123".to_string()));
    }

    #[test]
    fn test_generic_explorer_optional_urls() {
        let config = Metadata {
            name: "SimpleExplorer",
            base_url: "https://simple.com",
            tx_path: TRANSACTION_PATH,
            address_path: ACCOUNT_PATH,
            token_path: None,
            validator_path: None,
        };
        let explorer = Explorer::boxed(config);

        assert_eq!(explorer.get_token_url("token123"), None);
        assert_eq!(explorer.get_validator_url("val123"), None);
    }

    #[test]
    fn test_metadata_helpers() {
        let simple = Metadata::new("Simple", "https://simple.com");
        assert_eq!(simple.name, "Simple");
        assert_eq!(simple.base_url, "https://simple.com");
        assert_eq!(simple.tx_path, TX_PATH);
        assert_eq!(simple.address_path, ADDRESS_PATH);
        assert_eq!(simple.token_path, None);
        assert_eq!(simple.validator_path, None);

        let with_token = Metadata::with_token("WithToken", "https://token.com");
        assert_eq!(with_token.token_path, Some(TOKEN_PATH));
        assert_eq!(with_token.validator_path, None);

        let with_validator = Metadata::with_validator("WithValidator", "https://validator.com");
        assert_eq!(with_validator.token_path, None);
        assert_eq!(with_validator.validator_path, Some(VALIDATOR_PATH));

        let full = Metadata::full("Full", "https://full.com");
        assert_eq!(full.token_path, Some(TOKEN_PATH));
        assert_eq!(full.validator_path, Some(VALIDATOR_PATH));

        let transaction_style = Metadata::blockchair("Transaction", "https://transaction.com");
        assert_eq!(transaction_style.tx_path, TRANSACTION_PATH);
        assert_eq!(transaction_style.address_path, ADDRESS_PATH);
        assert_eq!(transaction_style.token_path, None);

        let cosmos_style = Metadata::mintscan("Cosmos", "https://cosmos.com");
        assert_eq!(cosmos_style.token_path, Some(ASSETS_PATH));
        assert_eq!(cosmos_style.validator_path, Some(VALIDATORS_PATH));
    }

    #[test]
    fn test_multi_chain_explorer() {
        let multi_explorer = MultiChainExplorer::new()
            .add_chain(
                "chain1",
                Metadata {
                    name: "MultiTest",
                    base_url: "https://chain1.com",
                    tx_path: TX_PATH,
                    address_path: ADDRESS_PATH,
                    token_path: None,
                    validator_path: None,
                },
            )
            .add_chain(
                "chain2",
                Metadata {
                    name: "MultiTest",
                    base_url: "https://chain2.com",
                    tx_path: TRANSACTION_PATH,
                    address_path: ACCOUNT_PATH,
                    token_path: Some(TOKEN_PATH),
                    validator_path: None,
                },
            );

        let chain1_explorer = multi_explorer.for_chain("chain1").unwrap();
        let chain2_explorer = multi_explorer.for_chain("chain2").unwrap();

        assert_eq!(chain1_explorer.get_tx_url("hash"), "https://chain1.com/tx/hash");
        assert_eq!(chain2_explorer.get_tx_url("hash"), "https://chain2.com/transaction/hash");
        assert_eq!(chain2_explorer.get_token_url("token"), Some("https://chain2.com/token/token".to_string()));

        assert!(multi_explorer.for_chain("nonexistent").is_none());
    }
}
