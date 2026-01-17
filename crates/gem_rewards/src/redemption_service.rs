use primitives::Asset;
use std::error::Error;

#[derive(Clone)]
pub struct RedemptionAsset {
    pub asset: Asset,
    pub value: String,
}

#[derive(Clone)]
pub struct RedemptionRequest {
    pub recipient_address: String,
    pub asset: Option<RedemptionAsset>,
}

pub struct RedemptionResult {
    pub transaction_id: String,
}

pub trait RedemptionService: Send + Sync {
    fn process_redemption(
        &self,
        request: RedemptionRequest,
    ) -> impl std::future::Future<Output = Result<RedemptionResult, Box<dyn Error + Send + Sync>>> + Send;
}
