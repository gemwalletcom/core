use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::{AssetId, Chain, TransactionType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanTransactionPayload {
    pub device_id: String,
    pub wallet_index: u32,
    pub origin: ScanAddressTarget,
    pub target: ScanAddressTarget,
    pub website: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanTransaction {
    pub is_malicious: bool,
    pub is_memo_required: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanAddressTarget {
    pub chain: Chain,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum AddressType {
    Address,
    Contract,
    Validator,
}

impl AddressType {
    pub fn all() -> Vec<AddressType> {
        AddressType::iter().collect::<Vec<_>>()
    }
}
