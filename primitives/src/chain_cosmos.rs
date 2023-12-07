#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum CosmosChain {
    Cosmos,
    Osmosis,
    Celestia,
    Thorchain,
}
