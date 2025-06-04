use alloy_ens::namehash;
use alloy_primitives::{Address, Bytes, U256};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::TransactionRequest;
use alloy_sol_types::{sol, SolCall};
use alloy_transport_http::Http;
use anyhow::{anyhow, Result};
use std::str::FromStr;
use url::Url;

sol! {
    #[sol(rpc)]
    interface ENSRegistry {
        function resolver(bytes32 node) external view returns (address);
    }

    #[sol(rpc)]
    interface ENSResolver {
        function addr(bytes32 node) external view returns (address);

        function addr_with_coin_type(bytes32 node, uint256 coin_type) external view returns (bytes);
    }
}

pub struct Contract {
    pub registry_address: Address,
    pub client: RpcClient,
}

impl Contract {
    pub fn new(rpc_url: &str, registry_address_hex: &str) -> Result<Self> {
        let url = Url::parse(rpc_url)?;
        let http_client = Http::new(url);
        let rpc_client = RpcClient::new(http_client, true);
        let registry_address = Address::from_str(registry_address_hex)?;
        Ok(Self {
            registry_address,
            client: rpc_client,
        })
    }
    pub async fn resolver(&self, name: &str) -> Result<Address> {
        let node = namehash(name);
        let call = ENSRegistry::resolverCall { node };
        let calldata = Bytes::from(call.abi_encode());

        let result_bytes = self.eth_call(self.registry_address, calldata).await?;
        if result_bytes.is_empty() || result_bytes.iter().all(|&b| b == 0) {
            return Err(anyhow!("No resolver set or resolver address is zero"));
        }
        // The result of resolver(bytes32) is an address, which is 20 bytes.
        // It might be padded to 32 bytes in the return data.
        // Address::decode expects exactly 20 bytes or a 32-byte slice where first 12 are zero.
        if result_bytes.len() == 32 && result_bytes[0..12].iter().all(|&b| b == 0) {
            Ok(Address::from_slice(&result_bytes[12..]))
        } else if result_bytes.len() == 20 {
            Ok(Address::from_slice(&result_bytes))
        } else {
            Err(anyhow!("Invalid resolver address format returned"))
        }
    }

    #[allow(unused)]
    pub async fn addr(&self, resolver_address_hex: &str, name: &str, coin_id: u32) -> Result<Bytes> {
        let node = namehash(name);
        let resolver_address = Address::from_str(resolver_address_hex)?;
        let call = ENSResolver::addr_with_coin_typeCall {
            node,
            coin_type: U256::from(coin_id),
        };
        let calldata = Bytes::from(call.abi_encode());

        self.eth_call(resolver_address, calldata).await
    }

    pub async fn legacy_addr(&self, resolver_address_hex: &str, name: &str) -> Result<Address> {
        let node = namehash(name);
        let resolver_address = Address::from_str(resolver_address_hex)?;
        let call = ENSResolver::addrCall { node };
        let calldata = Bytes::from(call.abi_encode());

        let result_bytes = self.eth_call(resolver_address, calldata).await?;
        if result_bytes.is_empty() || result_bytes.iter().all(|&b| b == 0) {
            return Err(anyhow!("No address found or address is zero"));
        }
        if result_bytes.len() == 32 && result_bytes[0..12].iter().all(|&b| b == 0) {
            Ok(Address::from_slice(&result_bytes[12..]))
        } else if result_bytes.len() == 20 {
            Ok(Address::from_slice(&result_bytes))
        } else {
            Err(anyhow!("Invalid address format returned"))
        }
    }

    async fn eth_call(&self, to: Address, data: Bytes) -> Result<Bytes> {
        let tx = TransactionRequest {
            to: Some(alloy_primitives::TxKind::Call(to)),
            input: data.into(), // Converts Bytes to rpc_types::Input
            ..Default::default()
        };
        let params = (tx, alloy_rpc_types::BlockId::latest());
        self.client.request("eth_call", params).await.map_err(|e| anyhow!(e))
    }
}
