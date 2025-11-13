use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};
use typeshare::typeshare;

use crate::{AssetId, Chain, TransactionType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanTransactionPayload {
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
    pub asset_id: AssetId,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AddressType {
    Address,
    Contract,
    Validator,
}

impl AddressType {
    pub fn all() -> Vec<Self> {
        Self::iter().collect::<Vec<_>>()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ScanAddress {
    pub chain: Chain,
    pub address: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub address_type: Option<AddressType>,
    pub is_malicious: Option<bool>,
    pub is_memo_required: Option<bool>,
    pub is_verified: Option<bool>,
}
