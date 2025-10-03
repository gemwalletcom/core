use std::error::Error;

#[cfg(feature = "rpc")]
use base64::{Engine as _, engine::general_purpose};
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "rpc")]
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
#[cfg(feature = "rpc")]
use num_bigint::BigInt;
use primitives::chain::Chain;
#[cfg(feature = "rpc")]
use serde::de::DeserializeOwned;
#[cfg(feature = "rpc")]
use sui_types::Address;

use crate::models::SuiCoinMetadata;
use crate::models::staking::{SuiStakeDelegation, SuiSystemState, SuiValidators};
use crate::models::transaction::{SuiBroadcastTransaction, SuiTransaction};
use crate::models::{Balance, Checkpoint, Digest, Digests, ResultData, TransactionBlocks};
#[cfg(feature = "rpc")]
use crate::models::{CoinAsset, InspectResult, SuiObject};
#[cfg(feature = "rpc")]
use crate::{
    SUI_COIN_TYPE, SUI_COIN_TYPE_FULL,
    jsonrpc::{SuiData, SuiRpc},
};
use primitives::transaction_load_metadata::SuiCoin;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
pub struct SuiClient {
    client: JsonRpcClient,
}

#[cfg(feature = "rpc")]
pub struct SuiClient<C: Client + Clone> {
    client: GenericJsonRpcClient<C>,
    pub chain: Chain,
}

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
impl SuiClient {
    pub fn new(client: JsonRpcClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> SuiClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Sui }
    }

    pub fn get_client(&self) -> &GenericJsonRpcClient<C> {
        &self.client
    }

    pub async fn rpc_call<T: DeserializeOwned + Clone>(&self, rpc: SuiRpc) -> Result<T, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request(rpc).await?)
    }

    pub async fn get_coin_assets(&self, owner: Address) -> Result<Vec<CoinAsset>, Box<dyn Error + Send + Sync>> {
        let mut coins: SuiData<Vec<CoinAsset>> = self.rpc_call(SuiRpc::GetAllCoins { owner: owner.to_string() }).await?;
        for coin in &mut coins.data {
            if coin.coin_type == SUI_COIN_TYPE {
                coin.coin_type = SUI_COIN_TYPE_FULL.into();
            }
        }
        Ok(coins.data)
    }

    pub async fn inspect_transaction_block(&self, sender: &str, tx_data: &[u8]) -> Result<InspectResult, Box<dyn Error + Send + Sync>> {
        let tx_bytes_base64 = general_purpose::STANDARD.encode(tx_data);
        self.rpc_call(SuiRpc::InspectTransactionBlock(sender.to_string(), tx_bytes_base64)).await
    }

    pub async fn get_balance(&self, address: String) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("suix_getBalance", serde_json::json!([address])).await?)
    }

    pub async fn get_all_balances(&self, address: String) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("suix_getAllBalances", serde_json::json!([address])).await?)
    }

    pub async fn get_stake_delegations(&self, address: String) -> Result<Vec<SuiStakeDelegation>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("suix_getStakes", serde_json::json!([address])).await?)
    }

    pub async fn get_validators(&self) -> Result<SuiValidators, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("suix_getValidatorsApy", serde_json::json!([])).await?)
    }

    pub async fn get_system_state(&self) -> Result<SuiSystemState, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("suix_getLatestSuiSystemState", serde_json::json!([])).await?)
    }

    pub async fn get_coin_metadata(&self, token_id: String) -> Result<SuiCoinMetadata, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("suix_getCoinMetadata", serde_json::json!([token_id])).await?)
    }

    pub async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.client.call("sui_getChainIdentifier", serde_json::json!([])).await?)
    }

    pub async fn get_latest_block(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let result = self
            .client
            .call::<String>("sui_getLatestCheckpointSequenceNumber", serde_json::json!([]))
            .await?;
        Ok(result.parse().unwrap_or(0))
    }

    pub async fn get_gas_price(&self) -> Result<BigInt, Box<dyn Error + Send + Sync>> {
        let result = self.client.call::<String>("suix_getReferenceGasPrice", serde_json::json!([])).await?;
        Ok(result.parse().unwrap_or(BigInt::from(1000)))
    }

    pub async fn get_coins(&self, address: &str, coin_type: &str) -> Result<Vec<SuiCoin>, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([address, coin_type, null, null]);
        Ok(self.client.call::<ResultData<Vec<SuiCoin>>>("suix_getCoins", params).await?.data)
    }

    pub async fn get_object(&self, object_id: String) -> Result<SuiObject, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([object_id, {"showContent": true}]);
        Ok(self.client.call::<ResultData<SuiObject>>("sui_getObject", params).await?.data)
    }

    pub async fn dry_run(&self, tx_data: String) -> Result<SuiTransaction, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([tx_data]);
        Ok(self.client.call("sui_dryRunTransactionBlock", params).await?)
    }

    pub async fn get_transaction(&self, transaction_id: String) -> Result<Digest, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([
            transaction_id,
            {
                "showInput": true,
                "showEffects": true,
                "showEvents": true,
                "showObjectChanges": true,
                "showBalanceChanges": true
            }
        ]);
        Ok(self.client.call("sui_getTransactionBlock", params).await?)
    }

    pub async fn get_transactions_by_address(&self, address: String) -> Result<TransactionBlocks, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([
            {
                "filter": {
                    "FromAddress": address
                },
                "options": {
                    "showInput": true,
                    "showEffects": true,
                    "showEvents": true,
                    "showObjectChanges": true,
                    "showBalanceChanges": true
                }
            }
        ]);
        Ok(self.client.call("suix_queryTransactionBlocks", params).await?)
    }

    pub async fn get_transactions_by_block(&self, checkpoint: u64) -> Result<Checkpoint, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([checkpoint.to_string()]);
        Ok(self.client.call("sui_getCheckpoint", params).await?)
    }

    pub async fn get_checkpoints(&self, checkpoint: u64, limit: Option<u64>) -> Result<Vec<Digest>, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([checkpoint.to_string(), limit.unwrap_or(1), false]);
        Ok(self.client.call::<Digests>("sui_getCheckpoints", params).await?.data)
    }

    pub async fn get_checkpoint_transactions(&self, checkpoint: u64, limit: Option<usize>) -> Result<TransactionBlocks, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([
            {
                "filter": {
                    "Checkpoint": checkpoint.to_string()
                },
                "options": {
                    "showInput": true,
                    "showEffects": true,
                    "showEvents": true,
                    "showObjectChanges": true,
                    "showBalanceChanges": true
                }
            },
            null,
            limit.unwrap_or(250),
            true
        ]);
        Ok(self.client.call("suix_queryTransactionBlocks", params).await?)
    }

    pub async fn broadcast(&self, data: String, signature: String) -> Result<SuiBroadcastTransaction, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([data, [signature], null, "WaitForLocalExecution"]);
        Ok(self.client.call("sui_executeTransactionBlock", params).await?)
    }
}
