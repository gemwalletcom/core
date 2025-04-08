use super::model::MayanTransactionResult;
use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use std::sync::Arc;

pub struct MayanExplorer {
    provider: Arc<dyn AlienProvider>,
}

impl MayanExplorer {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<MayanTransactionResult, SwapperError> {
        let url = format!("https://explorer-api.mayan.finance/v3/swap/trx/{}", tx_hash);
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        let result: MayanTransactionResult = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(result)
    }
}
