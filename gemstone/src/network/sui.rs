use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose};
use gem_sui::{
    SUI_COIN_TYPE, SUI_COIN_TYPE_FULL,
    jsonrpc::{SuiData, SuiRpc},
    models::{CoinAsset, InspectResult},
    rpc::client::SuiClient as GemSuiClient,
};
use primitives::Chain;
use serde::de::DeserializeOwned;
use sui_types::Address;

use super::{AlienClient, AlienError, AlienProvider, jsonrpc_client_with_chain};

pub struct SuiRpcClient {
    inner: GemSuiClient<AlienClient>,
}

impl SuiRpcClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Result<Self, AlienError> {
        let client = jsonrpc_client_with_chain(provider, Chain::Sui);
        Ok(Self {
            inner: GemSuiClient::new(client),
        })
    }

    pub fn inner(&self) -> &GemSuiClient<AlienClient> {
        &self.inner
    }

    pub async fn rpc_call<T: DeserializeOwned + Clone>(&self, rpc: SuiRpc) -> Result<T, AlienError> {
        self.inner.rpc_call(rpc).await.map_err(|e| AlienError::ResponseError { msg: e.to_string() })
    }

    pub async fn get_coin_assets(&self, owner: Address) -> Result<Vec<CoinAsset>, AlienError> {
        let mut coins: SuiData<Vec<CoinAsset>> = self.rpc_call(SuiRpc::GetAllCoins { owner: owner.to_string() }).await?;
        for coin in &mut coins.data {
            if coin.coin_type == SUI_COIN_TYPE {
                coin.coin_type = SUI_COIN_TYPE_FULL.into();
            }
        }
        Ok(coins.data)
    }

    pub async fn get_gas_price(&self) -> Result<u64, AlienError> {
        let gas_price: String = self.rpc_call(SuiRpc::GetGasPrice).await?;
        gas_price.parse::<u64>().map_err(|e| AlienError::ResponseError {
            msg: format!("Failed to parse gas price: {e}"),
        })
    }

    pub async fn inspect_transaction_block(&self, sender: &str, tx_data: &[u8]) -> Result<InspectResult, AlienError> {
        let tx_bytes_base64 = general_purpose::STANDARD.encode(tx_data);
        self.rpc_call(SuiRpc::InspectTransactionBlock(sender.to_string(), tx_bytes_base64)).await
    }
}
