// use chrono::{DateTime, Utc};
use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanTransactionResult {
    pub trader: String,
    pub source_tx_hash: String,
    pub deadline: String,
    pub source_chain: String,
    pub dest_chain: String,
    pub from_token_address: String,
    pub to_token_address: String,
    pub from_amount: String,
    pub to_amount: String,
    pub fulfill_tx_hash: Option<String>,
    pub refund_tx_hash: Option<String>,
    pub steps: Vec<MayanTransactionStep>,
    pub client_status: MayanClientStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MayanClientStatus {
    Completed,
    InProgress,
    Refunded,
}

impl MayanClientStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            MayanClientStatus::Completed => SwapStatus::Completed,
            MayanClientStatus::Refunded => SwapStatus::Refunded,
            MayanClientStatus::InProgress => SwapStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MayanTransactionStepStatus {
    Completed,
    Pending,
    Failed,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MayanTransactionStepType {
    Info,
    #[serde(alias = "BLOCK_COUNTER")]
    BlockCounter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanTransactionStep {
    pub title: String,
    pub status: MayanTransactionStepStatus,
    pub r#type: MayanTransactionStepType,
    pub meta: Option<MayanTransactionStepMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MayanTransactionStepMeta {
    pub start_block: u64,
    pub w_chain_id: u64,
    pub minimum_confirmation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_mayan_transaction_result() {
        let json = include_str!("../test/eth_to_sui_swift.json");
        let result: MayanTransactionResult = serde_json::from_str(json).unwrap();

        assert_eq!(result.steps[0].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[0].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.steps[1].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[1].r#type, MayanTransactionStepType::BlockCounter);
        assert_eq!(result.client_status, MayanClientStatus::Completed);

        let json = include_str!("../test/mctp_pending.json");
        let result: MayanTransactionResult = serde_json::from_str(json).unwrap();

        assert_eq!(result.steps[0].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[0].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.steps[1].status, MayanTransactionStepStatus::Active);
        assert_eq!(result.steps[1].r#type, MayanTransactionStepType::BlockCounter);
        assert_eq!(result.client_status, MayanClientStatus::InProgress);

        let json = include_str!("../test/swift_refunded.json");
        let result: MayanTransactionResult = serde_json::from_str(json).unwrap();

        assert_eq!(result.steps[0].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[0].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.steps[1].status, MayanTransactionStepStatus::Failed);
        assert_eq!(result.steps[1].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.client_status, MayanClientStatus::Refunded);
    }
}
