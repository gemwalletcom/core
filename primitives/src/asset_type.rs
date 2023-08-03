#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum AssetType {
    NATIVE,
    ERC20,
    BEP2,
    BEP20,
    SPL,
    ARBITRUM,
    TRC20,
}

#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum AssetSubtype {
    NATIVE,
    TOKEN,
}