#[typeshare(swift = "Codable")]
#[serde(rename_all = "camelCase")]
struct FiatAssets {
    version: u32,
    asset_ids: Vec<String>,
}