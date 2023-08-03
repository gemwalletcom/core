#[typeshare]
struct Asset {
    id: AssetId,
    name: String,
    symbol: String,
    decimals: Int,
    #[serde(rename = "type")]
    asset_type: AssetType
}