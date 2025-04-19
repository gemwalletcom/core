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
}
