#[typeshare]
struct CosmosDelegations {
    delegation_responses: Vec<CosmosDelegation>,
}

#[typeshare]
struct CosmosDelegation {
    balance: CosmosBalance,
}

#[typeshare]
struct CosmosUnboundingDelegations {
    unbonding_responses: Vec<CosmosUnboundingDelegation>,
}

#[typeshare]
struct CosmosUnboundingDelegation {
    entries: Vec<CosmosUnboudingDelegationEntry>,
}

#[typeshare]
struct CosmosUnboudingDelegationEntry {
    balance: String,
}

#[typeshare]
struct CosmosRewards {
    rewards: Vec<CosmosReward>,
}

#[typeshare]
struct CosmosReward {
    reward: Vec<CosmosBalance>,
}
