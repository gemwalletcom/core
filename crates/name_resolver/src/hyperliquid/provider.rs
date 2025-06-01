use alloy_ens::namehash;
use alloy_primitives::{Address, Bytes, TxKind};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockId, TransactionRequest};
use alloy_sol_types::SolCall;
use alloy_transport_http::Http;
use anyhow::Result;
use async_trait::async_trait;
use std::{error::Error, str::FromStr};
use url::Url;

use super::contracts::{Registrator, Router};
use crate::client::NameClient;
use primitives::{Chain, NameProvider};

const ROUTER_ADDRESS: &str = "0x25d1971d6dc9812ea1111662008f07735c74bff5";

#[derive(Debug)]
pub struct HyperliquidNames {
    client: RpcClient,
    resolver_address: Address,
}

impl HyperliquidNames {
    pub fn new(provider_url: String) -> Self {
        let url: Url = provider_url.parse().expect("Invalid HyperEVM node URL");
        let http_transport = Http::new(url);
        let client = RpcClient::new(http_transport, true);
        let resolver_address = Address::from_str(ROUTER_ADDRESS).expect("Invalid Router address");
        Self { client, resolver_address }
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
impl NameClient for HyperliquidNames {
    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Hyperliquid]
    }

    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        if !Self::is_valid_name(name) {
            return Err(format!("Invalid name: {}", name).into());
        }

        // Get current registrator
        let resolver_address = self.resolver_address;
        let call_data = Router::getCurrentRegistratorCall {}.abi_encode();
        let tx_request = TransactionRequest {
            to: Some(TxKind::Call(resolver_address)),
            input: Bytes::from(call_data).into(),
            ..Default::default()
        };

        let result = self
            .client
            .request::<(TransactionRequest, BlockId), Bytes>("eth_call", (tx_request, BlockId::latest()))
            .await?;

        // Get owner of name
        let node = namehash(name);
        let registrator = Router::getCurrentRegistratorCall::abi_decode_returns(&result)?.0;
        let call_data = Registrator::ownerOfCall { _namehash: node }.abi_encode();
        let tx_request = TransactionRequest {
            to: Some(TxKind::Call(Address::from(registrator))),
            input: Bytes::from(call_data).into(),
            ..Default::default()
        };
        let result = self
            .client
            .request::<(TransactionRequest, BlockId), Bytes>("eth_call", (tx_request, BlockId::latest()))
            .await?;

        let owner_address = Registrator::ownerOfCall::abi_decode_returns(&result)?.0;
        let address = Address::from(owner_address);

        Ok(address.to_checksum(None))
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
        assert!(HyperliquidNames::is_valid_name("test.hl"));
        assert!(HyperliquidNames::is_valid_name("a.test.hl"));
        assert!(HyperliquidNames::is_valid_name("a.b.test.hl"));
        assert!(HyperliquidNames::is_valid_name("foo-bar.hl"));
        assert!(HyperliquidNames::is_valid_name("123.hl"));

        assert!(!HyperliquidNames::is_valid_name("test..hl")); // Empty label
        assert!(!HyperliquidNames::is_valid_name("test.hl.")); // Trailing dot
        assert!(!HyperliquidNames::is_valid_name(".test.hl")); // Leading dot on label
        assert!(!HyperliquidNames::is_valid_name("test.hl.."));
        assert!(!HyperliquidNames::is_valid_name("test.hl..hl"));
        assert!(!HyperliquidNames::is_valid_name("")); // Empty name
        assert!(!HyperliquidNames::is_valid_name(".hl")); // Only TLD with leading dot
    }
}
