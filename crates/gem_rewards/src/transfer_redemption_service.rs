use crate::{RedemptionRequest, RedemptionResult, RedemptionService};
use std::error::Error;

pub struct TransferRedemptionService {
    _secret_phrase: String,
}

impl TransferRedemptionService {
    pub fn new(secret_phrase: String) -> Self {
        Self { _secret_phrase: secret_phrase }
    }
}

impl RedemptionService for TransferRedemptionService {
    async fn process_redemption(&self, request: RedemptionRequest) -> Result<RedemptionResult, Box<dyn Error + Send + Sync>> {
        let transaction_id = format!("pending_tx_{}", request.recipient_address);
        Ok(RedemptionResult { transaction_id })
    }
}
