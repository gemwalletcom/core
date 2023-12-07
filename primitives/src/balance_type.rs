#[typeshare(swift = "Equatable, Codable, CaseIterable")]
pub enum BalanceType {
    available,
    locked,
    frozen,
    staked,
    pending,
    rewards,
}