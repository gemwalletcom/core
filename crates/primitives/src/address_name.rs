use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Chain, VerificationStatus, scan::AddressType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct AddressName {
    pub chain: Chain,
    pub address: String,
    pub name: String,
    #[serde(rename = "type")]
    pub address_type: Option<AddressType>,
    pub status: VerificationStatus,
}
