use alloy_ens::namehash;
use alloy_primitives::{Address, Bytes, hex};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_jsonrpc::JsonRpcClient;
use serde_json::json;
use std::{error::Error, str::FromStr};

use super::{
    contracts::{Registrator, Router},
    record::Record,
};
use crate::{client::NameClient, ens::normalize_domain};
use primitives::{Chain, EVMChain, NameProvider};

const ROUTER_ADDRESS: &str = "0x25d1971d6dc9812ea1111662008f07735c74bff5";

pub struct Hyperliquid {
    client: JsonRpcClient<ReqwestClient>,
    router_address: Address,
}

impl Hyperliquid {
    pub fn new(provider_url: String) -> Self {
        let reqwest_client = gem_client::builder().build().expect("Failed to build reqwest client");
        let client = JsonRpcClient::new(ReqwestClient::new(provider_url, reqwest_client));
        let router_address = Address::from_str(ROUTER_ADDRESS).expect("Invalid Router address");

        Self {
            client,
            router_address,
        }
    }

    pub fn is_valid_name(name: &str) -> bool {
        let labels = name.split('.').collect::<Vec<&str>>();
        if labels.is_empty() {
            return false;
        }

        !labels.iter().any(|label| label.is_empty())
    }

    async fn eth_call(&self, to: Address, call_data: &[u8]) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let params = json!([
            {
                "to": to.to_string(),
                "data": hex::encode_prefixed(call_data)
            },
            "latest"
        ]);

        let result_str: String = self.client.call("eth_call", params).await?;
        let result = Bytes::from(hex::decode(&result_str)
            .map_err(|e| format!("Failed to decode hex response: {}", e))?);
        Ok(result)
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

        // Get current registrator from Router
        let router_call_data = Router::getCurrentRegistratorCall {}.abi_encode();
        let router_result = self.eth_call(self.router_address, &router_call_data).await?;
        let registrator = Router::getCurrentRegistratorCall::abi_decode_returns(&router_result)?.0;
        
        // Get full record JSON
        let registrator_call_data = Registrator::getFullRecordJSONCall { _namehash: node }.abi_encode();
        let registrator_result = self.eth_call(Address::from(registrator), &registrator_call_data).await?;
        let record_json = Registrator::getFullRecordJSONCall::abi_decode_returns(&registrator_result)?;
        let record: Record = serde_json::from_str(&record_json)?;
        
        // Get Resolved address for HyperEVM
        if chain == Chain::Hyperliquid {
            let resolved_address = &record.name.resolved;
            let address = Address::from_str(resolved_address)?;
            return Ok(address.to_checksum(None));
        }

        let slip44 = chain.as_slip44();
        let chain_address = record.data.chain_addresses.get(&slip44.to_string()).ok_or("Chain not found".to_string())?;
        Ok(match EVMChain::from_chain(chain) {
            Some(_) => Address::from_str(chain_address)?.to_checksum(None),
            None => chain_address.to_string(),
        })
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
