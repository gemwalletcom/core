use alloy_ens::namehash;
use alloy_primitives::{hex, Address, Bytes, U256};
use alloy_sol_types::SolCall;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use gem_jsonrpc::JsonRpcClient;
use serde_json::json;
use std::{error::Error, str::FromStr};

use super::{
    contracts::{Registrator, Router},
    record::Record,
};
use crate::{client::NameClient, ens::normalize_domain, hyperliquid::contracts::HyperliquidNames};
use primitives::{Chain, EVMChain, NameProvider};

const ROUTER_ADDRESS: &str = "0x25d1971d6dc9812ea1111662008f07735c74bff5";
const HYPERLIQUID_NAMES_ADDRESS: &str = "0x1d9d87eBc14e71490bB87f1C39F65BDB979f3cb7";

pub struct Hyperliquid {
    client: JsonRpcClient,
    router_address: Address,
    hyperliquid_names_address: Address,
}

impl Hyperliquid {
    pub fn new(provider_url: String) -> Self {
        let client = JsonRpcClient::new_reqwest(provider_url);
        let router_address = Address::from_str(ROUTER_ADDRESS).expect("Invalid Router address");
        let hyperliquid_names_address = Address::from_str(HYPERLIQUID_NAMES_ADDRESS).expect("Invalid Hyperliquid names address");
        Self {
            client,
            router_address,
            hyperliquid_names_address,
        }
    }

    pub fn is_valid_name(name: &str) -> bool {
        let labels = name.split('.').collect::<Vec<&str>>();
        if labels.is_empty() {
            return false;
        }

        !labels.iter().any(|label| label.is_empty())
    }
}

#[async_trait]
impl NameClient for Hyperliquid {
    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Bitcoin, Chain::Ethereum, Chain::Solana, Chain::Hyperliquid]
    }

    async fn resolve(&self, name: &str, chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        let name = normalize_domain(name)?;
        if !Self::is_valid_name(&name) {
            return Err(format!("Invalid name: {name}").into());
        }
        let node = namehash(&name);

        // Get Resolved address for HyperEVM
        if chain == Chain::Hyperliquid {
            let token_id = U256::from_be_bytes::<32>(node.as_slice().try_into().expect("node should be 32 bytes"));
            let call_data = HyperliquidNames::tokenIdToAddressCall { _tokenId: token_id }.abi_encode();
            let params = json!([
                {
                    "to": self.hyperliquid_names_address.to_string(),
                    "data": hex::encode_prefixed(&call_data)
                },
                "latest"
            ]);
            let result_str: String = self.client.call("eth_call", params).await?;
            let result = Bytes::from(hex::decode(&result_str).map_err(|e| anyhow!("Failed to decode hex response: {}", e))?);
            let address = HyperliquidNames::tokenIdToAddressCall::abi_decode_returns(&result)?.0;
            let address = Address::from(address);
            return Ok(address.to_checksum(None));
        }

        // Get Linked Address for other chains
        let router_address = self.router_address;
        let call_data = Router::getCurrentRegistratorCall {}.abi_encode();
        let params = json!([
            {
                "to": router_address.to_string(),
                "data": hex::encode_prefixed(&call_data)
            },
            "latest"
        ]);

        let result_str: String = self.client.call("eth_call", params).await?;
        let result = Bytes::from(hex::decode(&result_str).map_err(|e| anyhow!("Failed to decode hex response: {}", e))?);

        // Get full record json
        let registrator = Router::getCurrentRegistratorCall::abi_decode_returns(&result)?.0;
        let call_data = Registrator::getFullRecordJSONCall { _namehash: node }.abi_encode();
        let params = json!([
            {
                "to": Address::from(registrator).to_string(),
                "data": hex::encode_prefixed(&call_data)
            },
            "latest"
        ]);
        let result_str: String = self.client.call("eth_call", params).await?;
        let result = Bytes::from(hex::decode(&result_str).map_err(|e| anyhow!("Failed to decode hex response: {}", e))?);

        let record_json = Registrator::getFullRecordJSONCall::abi_decode_returns(&result)?;
        let record: Record = serde_json::from_str(&record_json)?;

        let slip44 = chain.as_slip44();
        let chain_address = record.data.chain_addresses.get(&slip44.to_string()).ok_or(anyhow!("Chain not found"))?;
        if EVMChain::from_chain(chain).is_some() {
            let address = Address::from_str(chain_address)?;
            return Ok(address.to_checksum(None));
        }
        Ok(chain_address.to_string())
    }

    fn provider(&self) -> NameProvider {
        NameProvider::Hyperliquid
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["hl"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_name() {
        assert!(Hyperliquid::is_valid_name("test.hl"));
        assert!(Hyperliquid::is_valid_name("a.test.hl"));
        assert!(Hyperliquid::is_valid_name("a.b.test.hl"));
        assert!(Hyperliquid::is_valid_name("foo-bar.hl"));
        assert!(Hyperliquid::is_valid_name("123.hl"));
        assert!(Hyperliquid::is_valid_name("🐈🐈🐈🐈🐈🐈🐈.hl"));

        assert!(!Hyperliquid::is_valid_name("test..hl")); // Empty label
        assert!(!Hyperliquid::is_valid_name("test.hl.")); // Trailing dot
        assert!(!Hyperliquid::is_valid_name(".test.hl")); // Leading dot on label
        assert!(!Hyperliquid::is_valid_name("test.hl.."));
        assert!(!Hyperliquid::is_valid_name("test.hl..hl"));
        assert!(!Hyperliquid::is_valid_name("")); // Empty name
        assert!(!Hyperliquid::is_valid_name(".hl")); // Only TLD with leading dot
    }
}
