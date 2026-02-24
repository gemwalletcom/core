use super::model::MayanTransactionResult;
use crate::{
    SwapperError,
    alien::{RpcProvider, Target},
};
use std::sync::Arc;

pub struct MayanExplorer {
    base_url: String,
    provider: Arc<dyn RpcProvider>,
}

impl MayanExplorer {
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider>) -> Self {
        Self { base_url, provider }
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<MayanTransactionResult, SwapperError> {
        let url = format!("{}/v3/swap/trx/{tx_hash}", self.base_url);
        let target = Target::get(&url);
        let response = self.provider.request(target).await?;
        let result: MayanTransactionResult = serde_json::from_slice(&response.data).map_err(SwapperError::from)?;

        Ok(result)
    }
}
