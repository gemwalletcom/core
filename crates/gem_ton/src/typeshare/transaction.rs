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
