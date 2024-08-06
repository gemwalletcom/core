#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum FeePriority {
    slow,
    normal,
    fast,
}
