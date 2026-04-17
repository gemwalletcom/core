use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum TransferDataOutputType {
    EncodedTransaction,
    Signature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum TransferDataOutputAction {
    /// Sign only; return the signed bytes.
    Sign,
    /// Sign, broadcast, return the broadcast id/hash.
    Send,
    /// Sign, broadcast, return the signed encoded transaction (e.g. TON Connect BOC).
    SignAndSend,
}
