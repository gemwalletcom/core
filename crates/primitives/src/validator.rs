use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{AddressType, Chain, ScanAddress};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct StakeValidator {
    pub id: String,
    pub name: String,
}

impl StakeValidator {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub fn as_scan_address(&self, chain: Chain) -> Option<ScanAddress> {
        if self.name.is_empty() {
            return None;
        }

        Some(ScanAddress {
            chain,
            address: self.id.clone(),
            name: Some(self.name.chars().take(128).collect()),
            address_type: Some(AddressType::Validator),
            is_malicious: Some(false),
            is_memo_required: Some(false),
            is_verified: Some(true),
        })
    }
}
