#[typeshare]
struct CosmosDelegations {
    delegation_responses: Vec<CosmosDelegation>,
}

#[typeshare]
struct CosmosDelegation {
    delegation: CosmosDelegationData,
    balance: CosmosBalance,
}

#[typeshare]
struct CosmosDelegationData {
    validator_address: String,
}

#[typeshare]
struct CosmosUnboundingDelegations {
    unbonding_responses: Vec<CosmosUnboundingDelegation>,
}

#[typeshare]
struct CosmosUnboundingDelegation {
    validator_address: String,
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
