use super::model::NearIntentsTransactionResult;
use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use std::sync::Arc;

const ONECLICK_API_BASE_URL: &str = "https://api.1click.io/v0";

pub struct OneClickApi {
    provider: Arc<dyn AlienProvider>,
}

impl OneClickApi {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_transaction_status(&self, deposit_address: &str) -> Result<NearIntentsTransactionResult, SwapperError> {
        let url = format!("{ONECLICK_API_BASE_URL}/status/{deposit_address}");
        let target = AlienTarget::get(&url);
        let response = self.provider.request(target).await?;
        let result: NearIntentsTransactionResult = serde_json::from_slice(&response).map_err(SwapperError::from)?;

        Ok(result)
    }
}
