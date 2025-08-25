use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResult {
    pub final_execution_status: String,
    pub transaction: BroadcastTransaction,
    pub transaction_outcome: TransactionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastTransaction {
    pub hash: String,
    pub signer_id: String,
    pub receiver_id: String,
    pub actions: Vec<TransactionAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAction {
    #[serde(rename = "Transfer")]
    pub transfer: Option<TransferAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAction {
    pub deposit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutcome {
    pub outcome: Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub tokens_burnt: BigUint,
}
