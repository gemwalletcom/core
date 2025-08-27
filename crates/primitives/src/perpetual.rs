use crate::{Asset, AssetId, PerpetualPosition, PerpetualProvider};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct Perpetual {
    pub id: String,
    pub name: String,
    pub provider: PerpetualProvider,
    pub asset_id: AssetId,
    pub identifier: String,
    pub price: f64,
    pub price_percent_change_24h: f64,
    pub open_interest: f64,
    pub volume_24h: f64,
    pub funding: f64,
    pub leverage: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRefStr, EnumString)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PerpetualDirection {
    Short,
    Long,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct PerpetualPositionData {
    pub perpetual: Perpetual,
    pub asset: Asset,
    pub position: PerpetualPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct PerpetualData {
    pub perpetual: Perpetual,
    pub asset: Asset,
    pub metadata: PerpetualMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct PerpetualPositionsSummary {
    pub positions: Vec<PerpetualPosition>,
    pub balance: PerpetualBalance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub struct PerpetualBalance {
    pub available: f64,
    pub reserved: f64,
    pub withdrawable: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PerpetualMetadata {
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct PerpetualConfirmData {
    pub direction: PerpetualDirection,
    pub asset: Asset,
    pub asset_index: i32,
    pub price: String,
    pub fiat_value: f64,
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum AccountDataType {
    Activate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
pub enum PerpetualType {
    Open(PerpetualConfirmData),
    Close(PerpetualConfirmData),
}
