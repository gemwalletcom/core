use crate::network::{jsonrpc::JsonRpcResult, jsonrpc_call, AlienError, AlienProvider};
use base64::{engine::general_purpose, Engine as _};
use gem_sui::jsonrpc::{SuiData, SuiRpc};

use primitives::Chain;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use sui_types::base_types::SuiAddress;

use super::models::{CoinAsset, CoinData, InspectResults};

pub struct SuiClient {
    provider: Arc<dyn AlienProvider>,
}

impl SuiClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn rpc_call<T: DeserializeOwned + Clone>(&self, rpc: SuiRpc) -> Result<T, AlienError> {
        let response: JsonRpcResult<T> = jsonrpc_call(&rpc, self.provider.clone(), &Chain::Sui).await?;
        let result = response.take()?;
        Ok(result)
    }

    pub async fn get_coin_assets(&self, owner: SuiAddress) -> Result<Vec<CoinAsset>, AlienError> {
        let coins: SuiData<Vec<CoinData>> = self.rpc_call(SuiRpc::GetAllCoins { owner: owner.to_string() }).await?;
        let assets = coins.data.into_iter().map(|coin| coin.into()).collect();
        Ok(assets)
    }

    pub async fn get_gas_price(&self) -> Result<u64, AlienError> {
        let gas_price: String = self.rpc_call(SuiRpc::GetGasPrice).await?;
        Ok(gas_price.parse().unwrap_or(0))
    }

    pub async fn estimate_gas_budget(&self, sender: &str, tx_data: &[u8]) -> Result<u64, AlienError> {
        let tx_bytes_base64 = general_purpose::STANDARD.encode(tx_data);
        let result: SuiData<InspectResults> = self.rpc_call(SuiRpc::InspectTransactionBlock(sender.to_string(), tx_bytes_base64)).await?;
        let effects = result.data.effects;
        // Extract the gas used from the results
        let gas_used = effects.gas_used.computation_cost + effects.gas_used.storage_cost - effects.gas_used.storage_rebate;

        // Add a buffer for safety (20%)
        let gas_with_buffer = (gas_used as f64 * 1.2) as u64;

        Ok(gas_with_buffer)
    }
}
