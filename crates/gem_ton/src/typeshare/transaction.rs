use serde::{Deserialize, Serialize};
use typeshare::typeshare;

type UInt64 = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonBroadcastTransaction {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransaction {
    pub transaction_id: TonTransactionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransactionId {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonMessageTransactions {
    pub transactions: Vec<TonTransactionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransactionMessage {
    pub hash: String,
    pub out_msgs: Vec<TonTransactionOutMessage>,
    pub description: Option<TonTransactionDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransactionOutMessage {
    pub hash: String,
    pub bounce: bool,
    pub bounced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonJettonToken {
    pub jetton_content: TonJettonTokenContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonJettonBalance {
    pub balance: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonJettonTokenContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub data: TonJettonTokenContentData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonJettonTokenContentData {
    pub name: String,
    pub symbol: String,
    pub decimals: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransactionDescription {
    pub action: Option<TonTransactionAction>,
    pub compute_ph: Option<TonTransactionComputePhase>,
    pub aborted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransactionAction {
    pub valid: Option<bool>,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TonTransactionComputePhase {
    pub success: Option<bool>,
    pub exit_code: Option<i32>,
}
