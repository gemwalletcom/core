use crate::models::{Account, AccountAccessKey, Block, BroadcastResult, GasPrice, GenesisConfig};
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainStaking, ChainToken, ChainTraits};
use gem_client::Client;
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcError};
use primitives::{Asset, Chain};
use serde_json::json;

#[derive(Debug)]
pub struct NearClient<C: Client + Clone> {
    client: JsonRpcClient<C>,
    pub chain: Chain,
}

impl<C: Client + Clone> NearClient<C> {
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Near }
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, JsonRpcError> {
        let params = json!({
            "request_type": "view_account",
            "finality": "final",
            "account_id": address
        });
        self.client.call("query", params).await
    }

    pub async fn get_account_access_key(&self, address: &str, public_key: &str) -> Result<AccountAccessKey, JsonRpcError> {
        let params = json!({
            "request_type": "view_access_key",
            "finality": "final",
            "account_id": address,
            "public_key": public_key
        });
        self.client.call("query", params).await
    }

    pub async fn get_latest_block(&self) -> Result<Block, JsonRpcError> {
        let params = json!({"finality": "final"});
        self.client.call("block", params).await
    }

    pub async fn get_gas_price(&self) -> Result<GasPrice, JsonRpcError> {
        let params = json!([null]);
        self.client.call("gas_price", params).await
    }

    pub async fn get_genesis_config(&self) -> Result<GenesisConfig, JsonRpcError> {
        let params = json!({});
        self.client.call("EXPERIMENTAL_genesis_config", params).await
    }

    pub async fn broadcast_transaction(&self, signed_tx_base64: &str) -> Result<BroadcastResult, JsonRpcError> {
        let params = json!({"signed_tx_base64": signed_tx_base64});
        self.client.call("send_tx", params).await
    }

    pub async fn get_transaction_status(&self, tx_hash: &str, sender_account_id: &str) -> Result<BroadcastResult, JsonRpcError> {
        let params = json!({
            "tx_hash": tx_hash,
            "sender_account_id": sender_account_id,
            "wait_until": "EXECUTED"
        });
        self.client.call("tx", params).await
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, JsonRpcError> {
        Err(JsonRpcError {
            code: -32000,
            message: "NEAR token queries not implemented".to_string(),
        })
    }
}

impl<C: Client + Clone> ChainProvider for NearClient<C> {
    fn get_chain(&self) -> Chain {
        self.chain
    }
}

impl<C: Client + Clone> ChainStaking for NearClient<C> {}
impl<C: Client + Clone> ChainPerpetual for NearClient<C> {}
impl<C: Client + Clone> ChainAddressStatus for NearClient<C> {}
impl<C: Client + Clone> ChainAccount for NearClient<C> {}
impl<C: Client + Clone> ChainToken for NearClient<C> {}
impl<C: Client + Clone> ChainTraits for NearClient<C> {}
