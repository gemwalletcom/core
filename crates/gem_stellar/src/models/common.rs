use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarEmbedded<T> {
    pub _embedded: StellarRecords<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarRecords<T> {
    pub records: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarAsset {
    pub asset_code: String,
    pub asset_issuer: String,
    pub contract_id: Option<String>,
}
