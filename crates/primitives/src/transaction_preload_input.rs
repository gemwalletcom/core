use crate::TransactionInputType;
use serde::{Deserialize, Serialize};
//use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
//#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionPreloadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
}
