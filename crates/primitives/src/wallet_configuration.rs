use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{ChainAddress, WalletId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletConfiguration {
    pub multi_signature_accounts: Vec<ChainAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[typeshare(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletConfigurationResult {
    pub wallet_id: WalletId,
    pub configuration: WalletConfiguration,
}
