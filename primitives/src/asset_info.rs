use typeshare::typeshare;
use serde::{Serialize, Deserialize};

use crate::Asset;

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfos {  
    pub asset: Asset,
    pub info: Option<AssetInfo>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {  
    pub website: String,
    pub links: Vec<AssetLink>,
}

#[typeshare]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetLink {
    pub name: String,
    pub url: String
}