#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum FeePriority {
    slow,
    normal,
    fast,
}

#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum FeeUnitType {
    satVb,
    satB,
}
