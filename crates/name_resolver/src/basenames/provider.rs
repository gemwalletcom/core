use alloy_ens::namehash;
use alloy_primitives::{Address, Bytes};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockId, TransactionRequest as EthCallTransactionRequest};
use alloy_sol_types::SolCall;
use alloy_transport_http::Http;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::error::Error;
use std::str::FromStr;
use url::Url;

use super::contract::L2Resolver;
use crate::client::NameClient;
use primitives::{chain::Chain, name::NameProvider};

const L2_RESOLVER_ADDRESS: &str = "0xC6d566A56A1aFf6508b41f6c90ff131615583BCD";

pub struct Basenames {
    client: RpcClient,
    resolver_address: Address,
    chain: Chain,
}

impl Basenames {
    pub fn new(provider_url: String) -> Self {
        let url: Url = provider_url.parse().expect("Invalid provider URL");
        let http_transport = Http::new(url);
        let client = RpcClient::new(http_transport, true);
        let resolver_address = Address::from_str(L2_RESOLVER_ADDRESS).expect("Invalid resolver address");
        Self {
            client,
            resolver_address,
            chain: Chain::Base,
        }
    }

    async fn get_address_from_resolver(&self, name: &str) -> Result<Address> {
        let node = namehash(name);
        let call_data = L2Resolver::addrCall { node }.abi_encode();

        let tx = EthCallTransactionRequest {
            to: Some(alloy_primitives::TxKind::Call(self.resolver_address)),
            input: Bytes::from(call_data).into(),
            ..Default::default()
        };

        let response_bytes: Bytes = self
            .client
            .request::<(EthCallTransactionRequest, BlockId), Bytes>("eth_call", (tx, BlockId::latest()))
            .await
            .map_err(|e| anyhow!("eth_call RPC request failed: {}", e))?;

        L2Resolver::addrCall::abi_decode_returns(response_bytes.as_ref()).map_err(|e| anyhow!("Failed to decode ABI returns for addr: {}", e))
    }
}

#[async_trait]
impl NameClient for Basenames {
    async fn resolve(&self, name: &str, _chain: Chain) -> Result<String, Box<dyn Error + Send + Sync>> {
        if !name.ends_with(".base.eth") {
            return Err(anyhow!("Name does not end with .base.eth").into());
        }

        match self.get_address_from_resolver(name).await {
            Ok(addr) => {
                if addr.is_zero() {
                    Err(anyhow!("Address not found").into())
                } else {
                    Ok(addr.to_checksum(None))
                }
            }
            Err(e) => Err(anyhow!(e).into()),
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
