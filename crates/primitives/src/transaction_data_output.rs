use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum TransferDataOutputType {
    EncodedTransaction,
    Signature,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum TransferDataOutputAction {
    Sign,
    Send,
}
