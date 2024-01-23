#[typeshare]
struct Balance {
    available: BigInt,
    frozen: BigInt,
    locked: BigInt,
    staked: BigInt,
    pending: BigInt,
    rewards: BigInt,
}