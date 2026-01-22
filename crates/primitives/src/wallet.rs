use crate::{Account, WalletType};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Default, Serialize, Deserialize, EnumString, AsRefStr, PartialEq)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WalletSource {
    Create,
    #[default]
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct Wallet {
    pub id: String,
    pub external_id: Option<String>,
    pub name: String,
    pub index: i32,
    #[serde(rename = "type")]
    pub wallet_type: WalletType,
    pub accounts: Vec<Account>,
    pub order: i32,
    pub is_pinned: bool,
    pub image_url: Option<String>,
    pub source: WalletSource,
}
