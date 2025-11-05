use crate::Account;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "lowercase")]
pub enum WalletSource {
    Create,
    Import,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct Wallet {
    pub id: String,
    pub name: String,
    pub index: i32,
    #[serde(rename = "type")]
    pub wallet_type: WalletType,
    pub accounts: Vec<Account>,
    pub order: i32,
    #[serde(rename = "isPinned")]
    pub is_pinned: bool,
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    pub source: WalletSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
#[allow(non_camel_case_types)]
pub enum WalletType {
    multicoin,
    single,
    private_key,
    view,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct WalletId {
    pub id: String,
}
