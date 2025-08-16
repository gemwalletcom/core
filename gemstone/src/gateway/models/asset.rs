use primitives::Asset;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAsset {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub asset_type: String,
}

impl From<Asset> for GemAsset {
    fn from(asset: Asset) -> Self {
        Self {
            id: asset.id.to_string(),
            name: asset.name,
            symbol: asset.symbol,
            decimals: asset.decimals,
            asset_type: asset.asset_type.as_ref().to_string(),
        }
    }
}
