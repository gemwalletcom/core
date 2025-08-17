use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearBroadcastResult {
    pub final_execution_status: String,
    pub transaction: NearBroadcastTransaction,
    pub transaction_outcome: NearTransactionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearBroadcastTransaction {
    pub hash: String,
    pub signer_id: String,
    pub receiver_id: String,
    pub actions: Vec<NearTransactionAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearTransactionAction {
    #[serde(rename = "Transfer")]
    pub transfer: Option<NearTransferAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearTransferAction {
    pub deposit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearTransactionOutcome {
    pub outcome: NearOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearOutcome {
    pub tokens_burnt: String,
}
