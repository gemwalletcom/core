use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WalletType {
    Multicoin,
    Single,
    PrivateKey,
    View,
}

impl WalletType {
    pub fn notification_priority(&self) -> u8 {
        match self {
            WalletType::Multicoin => 0,
            WalletType::Single => 1,
            WalletType::PrivateKey => 2,
            WalletType::View => 3,
        }
    }
}
