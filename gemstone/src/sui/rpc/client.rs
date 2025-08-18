use base64::{engine::general_purpose, Engine as _};
use gem_sui::{
    jsonrpc::{SuiData, SuiRpc},
    SUI_COIN_TYPE, SUI_COIN_TYPE_FULL,
};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use sui_types::Address;

use super::models::{CoinAsset, InspectResult};
use crate::network::{AlienClient, AlienError, AlienProvider};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::Chain;

pub struct SuiClient {
    client: JsonRpcClient<AlienClient>,
}

impl SuiClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        let endpoint = provider.get_endpoint(Chain::Sui).unwrap();
        let alien_client = AlienClient::new(endpoint.clone(), provider);
        Self {
            client: JsonRpcClient::new(endpoint, alien_client),
        }
    }

    pub async fn rpc_call<T: DeserializeOwned + Clone>(&self, rpc: SuiRpc) -> Result<T, AlienError> {
        let result: T = self.client.request(rpc).await.map_err(|e| AlienError::ResponseError { msg: e.to_string() })?;
        Ok(result)
    }

    pub async fn get_coin_assets(&self, owner: Address) -> Result<Vec<CoinAsset>, AlienError> {
        let coins: SuiData<Vec<CoinAsset>> = self.rpc_call(SuiRpc::GetAllCoins { owner: owner.to_string() }).await?;
        let coins = coins
            .data
            .into_iter()
            .map(|mut coin| {
                if coin.coin_type == SUI_COIN_TYPE {
                    coin.coin_type = SUI_COIN_TYPE_FULL.into();
                }
                coin
            })
            .collect();
        Ok(coins)
    }

    pub async fn get_gas_price(&self) -> Result<u64, AlienError> {
        let gas_price: String = self.rpc_call(SuiRpc::GetGasPrice).await?;
        gas_price.parse::<u64>().map_err(|e| AlienError::ResponseError {
            msg: format!("Failed to parse gas price: {e:?}"),
        })
    }

    pub async fn inspect_tx_block(&self, sender: &str, tx_data: &[u8]) -> Result<InspectResult, AlienError> {
        let tx_bytes_base64 = general_purpose::STANDARD.encode(tx_data);
        let result: InspectResult = self.rpc_call(SuiRpc::InspectTransactionBlock(sender.to_string(), tx_bytes_base64)).await?;

        if result.error.is_some() {
            return Err(AlienError::ResponseError {
                msg: format!("Failed to inspect transaction: {:?}", result.error),
            });
        }
        Ok(result)
    }
}
