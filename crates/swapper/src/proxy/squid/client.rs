use super::model::SquidTransactionStatus;
use crate::{
    SwapperError,
    alien::{RpcProvider, Target},
};
use std::sync::Arc;

pub struct SquidClient {
    base_url: String,
    provider: Arc<dyn RpcProvider>,
}

impl SquidClient {
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider>) -> Self {
        Self { base_url, provider }
    }

    pub async fn get_transaction_status(&self, tx_hash: &str) -> Result<SquidTransactionStatus, SwapperError> {
        let url = format!("{}/v2/status?transactionId={tx_hash}", self.base_url);
        let target = Target::get(&url);
        let response = self.provider.request(target).await?;
        let result: SquidTransactionStatus = serde_json::from_slice(&response.data).map_err(SwapperError::from)?;
        Ok(result)
    }
}
