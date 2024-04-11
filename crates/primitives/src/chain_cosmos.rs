#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum CosmosChain {
    Cosmos,
    Osmosis,
    Celestia,
    Thorchain,
    Injective,
    Sei,
    Noble,
    Dymension,
    Saga,
}

#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum CosmosDenom {
    rune,
    uatom,
    uosmo,
    utia,
    inj,
    usei,
    uusdc,
    adym,
    usaga,
}
