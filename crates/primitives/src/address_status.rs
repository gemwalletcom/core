use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub enum AddressStatus {
    MultiSignature,
}