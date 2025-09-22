use alloy_ens::namehash;
use alloy_primitives::{hex, Address, Bytes};
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_jsonrpc::JsonRpcClient;
use serde_json::json;
use std::error::Error;
use std::str::FromStr;

use super::contract::L2Resolver;
use crate::client::NameClient;
use primitives::{chain::Chain, name::NameProvider};

const L2_RESOLVER_ADDRESS: &str = "0xC6d566A56A1aFf6508b41f6c90ff131615583BCD";

pub struct Basenames {
    client: JsonRpcClient<ReqwestClient>,
    resolver_address: Address,
    chain: Chain,
}

impl Basenames {
    pub fn new(provider_url: String) -> Self {
        let client = JsonRpcClient::new(ReqwestClient::new(provider_url, reqwest::Client::new()));
        let resolver_address = Address::from_str(L2_RESOLVER_ADDRESS).expect("Invalid resolver address");
        Self {
            client,
            resolver_address,
            chain: Chain::Base,
        }
    }

    async fn get_address_from_resolver(&self, name: &str) -> Result<Address, Box<dyn Error + Send + Sync>> {
        let node = namehash(name);
        let call_data = L2Resolver::addrCall { node }.abi_encode();

        let params = json!([
            {
                "to": self.resolver_address.to_string(),
                "data": hex::encode_prefixed(&call_data)
            },
            "latest"
        ]);

        let result: String = self.client.call("eth_call", params).await?;
        let response_bytes = Bytes::from(hex::decode(&result)?);

        L2Resolver::addrCall::abi_decode_returns(response_bytes.as_ref()).map_err(Into::into)
    }
}

#[async_trait]
impl NameClient for Basenames {
    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        match self.get_address_from_resolver(name).await {
            Ok(addr) => {
                if addr.is_zero() {
                    Err("Address not found".into())
                } else {
                    Ok(addr.to_checksum(None))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn provider(&self) -> NameProvider {
        NameProvider::Basenames
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["base.eth"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![self.chain]
    }
}
