use std::error::Error;

#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "rpc")]
use num_bigint::BigInt;
use primitives::chain::Chain;

use crate::models::staking::{SuiStakeDelegation, SuiSystemState, SuiValidators};
use crate::models::transaction::SuiBroadcastTransaction;
use crate::models::SuiCoinMetadata;
use crate::models::{Balance, Checkpoint, Digest, Digests, TransactionBlocks};

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
