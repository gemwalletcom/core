#[typeshare(swift = "Sendable")]
struct CosmosDelegations {
    delegation_responses: Vec<CosmosDelegation>,
}

#[typeshare(swift = "Sendable")]
struct CosmosDelegation {
    delegation: CosmosDelegationData,
    balance: CosmosBalance,
}

#[typeshare(swift = "Sendable")]
struct CosmosDelegationData {
    validator_address: String,
}

#[typeshare(swift = "Sendable")]
struct CosmosUnboundingDelegations {
    unbonding_responses: Vec<CosmosUnboundingDelegation>,
}

#[typeshare(swift = "Sendable")]
struct CosmosUnboundingDelegation {
    validator_address: String,
    entries: Vec<CosmosUnboudingDelegationEntry>,
}

#[typeshare(swift = "Sendable")]
struct CosmosUnboudingDelegationEntry {
    completion_time: String,
    creation_height: String,
    balance: String,
}

#[typeshare(swift = "Sendable")]
struct CosmosRewards {
    rewards: Vec<CosmosReward>,
}

#[typeshare(swift = "Sendable")]
struct CosmosReward {
    validator_address: String,
    reward: Vec<CosmosBalance>,
}

#[typeshare(swift = "Sendable")]
struct CosmosValidators {
    validators: Vec<CosmosValidator>,
}

#[typeshare(swift = "Sendable")]
struct CosmosValidator {
    operator_address: String,
    jailed: bool,
    status: String,
    description: CosmosValidatorMoniker,
    commission: CosmosValidatorCommission,
}

#[typeshare(swift = "Sendable")]
struct CosmosValidatorMoniker {
    moniker: String,
}

#[typeshare(swift = "Sendable")]
struct CosmosValidatorCommission {
    commission_rates: CosmosValidatorCommissionRates,
}

#[typeshare(swift = "Sendable")]
struct CosmosValidatorCommissionRates {
    rate: String,
}
