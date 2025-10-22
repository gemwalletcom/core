use super::model::MayanTransactionResult;
use crate::{
    SwapperError,
    alien::{RpcProvider, Target},
};
use std::sync::Arc;

pub struct MayanExplorer {
    provider: Arc<dyn RpcProvider>,
}

impl MayanExplorer {
    pub fn new(provider: Arc<dyn RpcProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<MayanTransactionResult, SwapperError> {
        let url = format!("https://explorer-api.mayan.finance/v3/swap/trx/{tx_hash}");
        let target = Target::get(&url);
        let response = self.provider.request(target).await?;
        let result: MayanTransactionResult = serde_json::from_slice(&response.data).map_err(SwapperError::from)?;

        Ok(result)
    }
}
