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
    completion_time: String,
    unbonding_id: String,
    balance: String,
}

#[typeshare]
struct CosmosRewards {
    rewards: Vec<CosmosReward>,
}

#[typeshare]
struct CosmosReward {
    validator_address: String,
    reward: Vec<CosmosBalance>,
}

#[typeshare]
struct CosmosValidators {
    validators: Vec<CosmosValidator>,
}

#[typeshare]
struct CosmosValidator {
    operator_address: String,
    jailed: bool,
    description: CosmosValidatorMoniker,
}

#[typeshare]
struct CosmosValidatorMoniker {
    moniker: String,
}
