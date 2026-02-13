#[typeshare(swift = "Equatable, CaseIterable, Sendable")]
pub enum BalanceType {
    available,
    locked,
    frozen,
    staked,
    pending,
    pendingUnconfirmed,
    rewards,
    reserved,
    earn,
}
