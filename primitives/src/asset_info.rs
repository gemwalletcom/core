use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
#[typeshare(swift = "Equatable, Codable, Hashable")]
pub struct AssetInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub r#type: String,
    
    pub status: String,
    pub website: String,
    pub description: String,
    
    pub tags: Option<Vec<String>>,
    pub links: Option<Vec<AssetInfoLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
#[typeshare(swift = "Equatable, Codable, Hashable")]
pub struct AssetInfoLink {
    pub name: String,
    pub url: String
}