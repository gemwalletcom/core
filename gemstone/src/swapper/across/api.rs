use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
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
    pub async fn deposit_status(&self, chain_id: &str, deposit_id: &str) -> Result<FillStatus, SwapperError> {
        let url = format!("{}/deposit/status?originChainId={}&depositId={}", self.url, chain_id, deposit_id);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        let status: FillStatus = serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;

        Ok(status)
    }
}
