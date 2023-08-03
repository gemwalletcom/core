#[typeshare]
#[typeshare(swift = "Equatable, Codable, Hashable")]
struct AssetId {
    chain: Chain,
    #[serde(rename = "tokenId")]
    token_id: Option<String>,
}