#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum TransactionDirection {
    #[serde(rename = "self")]
    self_transfer,
    outgoing,
    incoming,
}
