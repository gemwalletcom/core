use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::{eth_rpc, SwapperError},
};
use primitives::Chain;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AcrossApi {
    pub url: String,
    pub provider: Arc<dyn AlienProvider>,
}

impl AcrossApi {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            url: "https://app.across.to".into(),
            provider,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillStatus {
    pub fill_status: String,
    pub fill_tx_hash: String,
    pub destination_chain_id: u32,
}

impl FillStatus {
    pub fn is_filled(&self) -> bool {
        self.fill_status == "FILLED"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableRoute {
    pub origin_chain_id: u32,
    pub origin_token: String,
    pub destination_chain_id: u32,
    pub destination_token: String,
    pub is_native: bool,
}

impl AcrossApi {
    pub async fn deposit_status(&self, chain: Chain, tx_hash: &str) -> Result<FillStatus, SwapperError> {
        let receipt = eth_rpc::fetch_tx_receipt(self.provider.clone(), chain, tx_hash).await?;
        if receipt.logs.len() < 2 || receipt.logs[1].topics.len() < 4 {
            return Err(SwapperError::NetworkError("invalid tx receipt".into()));
        }
        let deposit_id = receipt.logs[1].topics[3].clone();
        let url = format!("{}/deposit/status?originChainId={}&depositId={}", self.url, chain.network_id(), &deposit_id);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        let status: FillStatus = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(status)
    }

    // https://across.to/api/available-routes
    pub async fn available_routes(&self) -> Result<Vec<AvailableRoute>, SwapperError> {
        let url = format!("{}/api/available-routes", self.url);
        let target = AlienTarget::get(&url).set_cache_ttl(24 * 60 * 60);
        let response = self.provider.request(target).await?;
        let routes: Vec<AvailableRoute> = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(routes)
    }
}
