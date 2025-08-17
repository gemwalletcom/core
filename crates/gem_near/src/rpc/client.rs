use std::error::Error;

use chain_traits::{ChainAccount, ChainPerpetual, ChainStaking, ChainToken, ChainTraits};
use gem_client::Client;
use primitives::{Asset, Chain, JsonRpcResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

#[derive(Debug)]
pub struct NearClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> NearClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Near }
    }

    pub async fn get_near_account(&self, address: &str) -> Result<crate::models::NearAccount, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "query".to_string(),
            params: json!({
                "request_type": "view_account",
                "finality": "final",
                "account_id": address
            }),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearAccount> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }

    pub async fn get_near_account_access_key(
        &self,
        address: &str,
        public_key: &str,
    ) -> Result<crate::models::NearAccountAccessKey, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "query".to_string(),
            params: json!({
                "request_type": "view_access_key",
                "finality": "final",
                "account_id": address,
                "public_key": public_key
            }),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearAccountAccessKey> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }

    pub async fn get_near_latest_block(&self) -> Result<crate::models::NearBlock, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "block".to_string(),
            params: json!({"finality": "final"}),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearBlock> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }

    pub async fn get_near_gas_price(&self) -> Result<crate::models::NearGasPrice, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "gas_price".to_string(),
            params: json!([null]),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearGasPrice> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }

    pub async fn get_near_genesis_config(&self) -> Result<crate::models::NearGenesisConfig, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "EXPERIMENTAL_genesis_config".to_string(),
            params: json!({}),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearGenesisConfig> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }

    pub async fn broadcast_near_transaction(&self, signed_tx_base64: &str) -> Result<crate::models::NearBroadcastResult, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "send_tx".to_string(),
            params: json!({"signed_tx_base64": signed_tx_base64}),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearBroadcastResult> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }

    pub async fn get_near_transaction_status(
        &self,
        tx_hash: &str,
        sender_account_id: &str,
    ) -> Result<crate::models::NearBroadcastResult, Box<dyn Error + Send + Sync>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "tx".to_string(),
            params: json!({
                "tx_hash": tx_hash,
                "sender_account_id": sender_account_id,
                "wait_until": "EXECUTED"
            }),
            id: 1,
        };
        let response: JsonRpcResult<crate::models::NearBroadcastResult> = self.client.post("", &request, None).await?;
        Ok(response.result)
    }
}

impl<C: Client> NearClient<C> {
    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Err("NEAR token queries not implemented".into())
    }
}

impl<C: Client> ChainStaking for NearClient<C> {}
impl<C: Client> ChainPerpetual for NearClient<C> {}
impl<C: Client> ChainAccount for NearClient<C> {}
impl<C: Client> ChainToken for NearClient<C> {}
impl<C: Client> ChainTraits for NearClient<C> {}
