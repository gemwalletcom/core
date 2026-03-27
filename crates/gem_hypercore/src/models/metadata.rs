use primitives::{AssetId, Chain};
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
    #[serde(rename = "onlyIsolated")]
    pub is_isolated_only: Option<bool>,
}

impl UniverseAsset {
    pub fn asset_id(&self) -> AssetId {
        perpetual_asset_id(&self.name)
    }

    pub fn asset_id_by_dex(&self, perp_dex_index: u32) -> AssetId {
        perpetual_asset_id_by_dex(&self.name, perp_dex_index)
    }
}

pub fn perpetual_asset_id(coin: &str) -> AssetId {
    let token_id = AssetId::sub_token_id(&["perpetual".to_string(), coin.to_string()]);
    AssetId::from(Chain::HyperCore, Some(token_id))
}

pub fn perpetual_asset_id_by_dex(coin: &str, perp_dex_index: u32) -> AssetId {
    if perp_dex_index == 0 {
        return perpetual_asset_id(coin);
    }

    let token_id = AssetId::sub_token_id(&["perpetual".to_string(), perp_dex_index.to_string(), coin.to_string()]);
    AssetId::from(Chain::HyperCore, Some(token_id))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id() {
        let asset = UniverseAsset {
            name: "BTC".to_string(),
            sz_decimals: 5,
            max_leverage: 50,
            is_isolated_only: None,
        };
        let asset_id = asset.asset_id();

        assert_eq!(asset_id.chain, Chain::HyperCore);
        assert_eq!(asset_id.token_id, Some("perpetual::BTC".to_string()));
    }

    #[test]
    fn test_asset_id_by_dex() {
        let asset = UniverseAsset {
            name: "BTC".to_string(),
            sz_decimals: 5,
            max_leverage: 50,
            is_isolated_only: None,
        };
        let asset_id = asset.asset_id_by_dex(2);

        assert_eq!(asset_id.chain, Chain::HyperCore);
        assert_eq!(asset_id.token_id, Some("perpetual::2::BTC".to_string()));
    }
}
