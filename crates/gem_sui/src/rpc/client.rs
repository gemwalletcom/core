use std::error::Error;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainPerpetual, ChainTraits, ChainTransactions, ChainPreload};
use primitives::{chain::Chain, Asset, JsonRpcResult, TransactionStateRequest, TransactionUpdate};

use super::{
    model::Balance,
};
use crate::models::staking::{SuiStakeDelegation, SuiSystemState, SuiValidators};
use crate::models::SuiCoinMetadata;
use crate::models::transaction::SuiTransaction;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
pub struct SuiClient {
    client: JsonRpcClient,
}

#[cfg(feature = "rpc")]
pub struct SuiClient<C: Client> {
    pub client: C,
}

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
impl SuiClient {
    pub fn new(client: JsonRpcClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> SuiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
    
    pub fn get_client(&self) -> &C {
        &self.client
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    async fn rpc_call<T>(&self, method: &str, params: serde_json::Value) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        let response: JsonRpcResult<T> = self.client.post("", &rpc_request, None).await?;
        Ok(response.result)
    }

    pub async fn get_balance(&self, address: String) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        self.rpc_call("suix_getBalance", serde_json::json!([address])).await
    }

    pub async fn get_all_balances(&self, address: String) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        self.rpc_call("suix_getAllBalances", serde_json::json!([address])).await
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let metadata = self.get_coin_metadata(token_id.clone()).await?;
        let asset_id = primitives::AssetId::from_token(Chain::Sui, &token_id);
        
        Ok(Asset {
            id: asset_id.clone(),
            name: metadata.name,
            symbol: metadata.symbol,
            decimals: metadata.decimals,
            chain: asset_id.chain,
            token_id: asset_id.token_id,
            asset_type: primitives::AssetType::TOKEN,
        })
    }

    pub async fn get_stake_delegations(&self, address: String) -> Result<Vec<SuiStakeDelegation>, Box<dyn Error + Send + Sync>> {
        self.rpc_call("suix_getStakes", serde_json::json!([address])).await
    }

    pub async fn get_validators(&self) -> Result<SuiValidators, Box<dyn Error + Send + Sync>> {
        self.rpc_call("suix_getValidatorsApy", serde_json::json!([])).await
    }

    pub async fn get_system_state(&self) -> Result<SuiSystemState, Box<dyn Error + Send + Sync>> {
        self.rpc_call("suix_getLatestSuiSystemState", serde_json::json!([])).await
    }

    pub async fn get_coin_metadata(&self, token_id: String) -> Result<SuiCoinMetadata, Box<dyn Error + Send + Sync>> {
        self.rpc_call("suix_getCoinMetadata", serde_json::json!([token_id])).await
    }

    pub async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.rpc_call("sui_getChainIdentifier", serde_json::json!([])).await
    }

    pub async fn get_latest_block(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let result: String = self.rpc_call("sui_getLatestCheckpointSequenceNumber", serde_json::json!([])).await?;
        Ok(result.parse().unwrap_or(0))
    }

    pub fn is_token_address(&self, token_id: &str) -> bool {
        token_id.starts_with("0x") && token_id.len() >= 66 && token_id.len() <= 100
    }

    pub async fn get_gas_price(&self) -> Result<num_bigint::BigInt, Box<dyn Error + Send + Sync>> {
        let result: String = self.rpc_call("suix_getReferenceGasPrice", serde_json::json!([])).await?;
        Ok(result.parse().unwrap_or(num_bigint::BigInt::from(1000)))
    }

    pub async fn get_transaction(&self, transaction_id: String) -> Result<SuiTransaction, Box<dyn Error + Send + Sync>> {
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
        self.rpc_call("sui_getTransactionBlock", params).await
    }

}

#[cfg(feature = "rpc")]
impl<C: Client> ChainTraits for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainAccount for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainPerpetual for SuiClient<C> {}



#[cfg(feature = "rpc")]
impl<C: Client> ChainPreload for SuiClient<C> {}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client> ChainTransactions for SuiClient<C> {
    async fn transaction_broadcast(&self, _data: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn std::error::Error + Sync + Send>> {
        let transaction = self.get_transaction(request.id).await?;
        let state = match transaction.effects.status.status.as_str() {
            "success" => primitives::TransactionState::Confirmed,
            "failure" => primitives::TransactionState::Reverted,
            _ => primitives::TransactionState::Pending,
        };
        Ok(TransactionUpdate::new_state(state))
    }
}
