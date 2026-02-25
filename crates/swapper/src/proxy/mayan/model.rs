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
    pub from_token_chain: String,
    pub to_token_chain: String,
    pub from_amount: String,
    pub to_amount: String,
    pub from_amount64: Option<String>,
    pub to_amount64: Option<String>,
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
            MayanClientStatus::Refunded => SwapStatus::Failed,
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
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/eth_to_sui_swift.json")).unwrap();
        assert_eq!(result.steps[0].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[0].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.steps[1].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[1].r#type, MayanTransactionStepType::BlockCounter);
        assert_eq!(result.client_status, MayanClientStatus::Completed);

        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/mctp_pending.json")).unwrap();
        assert_eq!(result.steps[0].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[0].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.steps[1].status, MayanTransactionStepStatus::Active);
        assert_eq!(result.steps[1].r#type, MayanTransactionStepType::BlockCounter);
        assert_eq!(result.client_status, MayanClientStatus::InProgress);

        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/swift_refunded.json")).unwrap();
        assert_eq!(result.steps[0].status, MayanTransactionStepStatus::Completed);
        assert_eq!(result.steps[0].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.steps[1].status, MayanTransactionStepStatus::Failed);
        assert_eq!(result.steps[1].r#type, MayanTransactionStepType::Info);
        assert_eq!(result.client_status, MayanClientStatus::Refunded);
    }

    #[test]
    fn test_token_chain_fields_eth_to_sui() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/eth_to_sui_swift.json")).unwrap();
        assert_eq!(result.from_token_chain, "2");
        assert_eq!(result.to_token_chain, "21");
        assert_eq!(result.from_token_address, "0x0000000000000000000000000000000000000000");
        assert_eq!(result.to_token_address, "0x2::sui::SUI");
    }

    #[test]
    fn test_token_chain_fields_base_to_arb() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/mctp_pending.json")).unwrap();
        assert_eq!(result.from_token_chain, "30");
        assert_eq!(result.to_token_chain, "23");
        assert_eq!(result.from_token_address, "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913");
        assert_eq!(result.to_token_address, "0xaf88d065e77c8cc2239327c5edb3a432268e5831");
    }

    #[test]
    fn test_token_chain_fields_refunded() {
        let result: MayanTransactionResult = serde_json::from_str(include_str!("../test/swift_refunded.json")).unwrap();
        assert_eq!(result.from_token_chain, "2");
        assert_eq!(result.to_token_chain, "1");
    }
}
