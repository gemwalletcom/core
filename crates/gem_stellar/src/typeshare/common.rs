use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
pub struct StellarEmbedded<T> {
    pub _embedded: StellarRecords<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
pub struct StellarRecords<T> {
    pub records: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
pub struct StellarAsset {
    pub asset_code: String,
    pub asset_issuer: String,
    pub contract_id: Option<String>,
}