use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMetadata {
    pub funding: String,
    pub open_interest: String,
    pub prev_day_px: String,
    pub day_ntl_vlm: String,
    pub premium: Option<String>,
    pub oracle_px: String,
    pub mark_px: String,
    pub mid_px: Option<String>,
    pub impact_pxs: Option<Vec<String>>,
    pub day_base_vlm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseAsset {
    pub name: String,
    pub sz_decimals: i32,
    pub max_leverage: i32,
    pub only_isolated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreUniverseResponse {
    pub universe: Vec<UniverseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypercoreMetadataResponse(pub HypercoreUniverseResponse, pub Vec<AssetMetadata>);

impl HypercoreMetadataResponse {
    pub fn universe(&self) -> &HypercoreUniverseResponse {
        &self.0
    }

    pub fn asset_metadata(&self) -> &Vec<AssetMetadata> {
        &self.1
    }
}

