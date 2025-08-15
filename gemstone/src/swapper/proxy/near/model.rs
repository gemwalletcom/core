use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearIntentsTransactionResult {
    pub status: NearIntentsTransactionStatus,
}

impl NearIntentsTransactionResult {
    pub fn swap_status(&self) -> SwapStatus {
        match self.status {
            NearIntentsTransactionStatus::Success => SwapStatus::Completed,
            NearIntentsTransactionStatus::Refunded => SwapStatus::Refunded,
            NearIntentsTransactionStatus::Pending => SwapStatus::Pending,
            NearIntentsTransactionStatus::Failed => SwapStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NearIntentsTransactionStatus {
    Success,
    Pending,
    Failed,
    Refunded,
}
