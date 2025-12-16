use primitives::AssetId;
use std::error::Error;

pub struct RedemptionAsset {
    pub asset_id: AssetId,
    pub amount: String,
}

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
