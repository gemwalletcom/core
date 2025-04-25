#[typeshare(swift = "Sendable")]
struct TronAccountRequest {
    address: String,
    visible: bool,
}

#[typeshare(swift = "Sendable")]
struct TronAccount {
    balance: Option<UInt64>,
    address: Option<String>,
    active_permission: Option<Vec<TronAccountPermission>>,
    votes: Option<Vec<TronVote>>,
    #[serde(rename = "frozenV2")]
    frozen_v2: Option<Vec<TronFrozen>>,
    #[serde(rename = "unfrozenV2")]
    unfrozen_v2: Option<Vec<TronUnfrozen>>,
}

#[typeshare(swift = "Sendable")]
struct TronAccountPermission {
    threshold: i32,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct TronAccountUsage {
    free_net_used: Option<i32>,
    free_net_limit: Option<i32>,
    #[serde(rename = "EnergyUsed")]
    energy_used: Option<UInt64>,
    #[serde(rename = "EnergyLimit")]
    energy_limit: Option<UInt64>,
}

#[typeshare(swift = "Sendable")]
struct TronEmptyAccount {
    address: Option<String>,
}

#[typeshare(swift = "Sendable")]
struct TronVote {
    vote_address: String,
    vote_count: UInt64,
}

#[typeshare(swift = "Sendable")]
struct TronFrozen {
    #[serde(rename = "type")]
    frozen_type: Option<String>,
    amount: Option<UInt64>,
}

#[typeshare(swift = "Sendable")]
struct TronUnfrozen {
    unfreeze_amount: Option<UInt64>,
    unfreeze_expire_time: Option<UInt64>,
}

#[typeshare(swift = "Sendable")]
struct TronReward {
    reward: Option<UInt64>,
}

#[typeshare(swift = "Sendable")]
struct WitnessesList {
    witnesses: Vec<WitnessAccount>,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct WitnessAccount {
    address: String,
    vote_count: Option<UInt64>,
    url: String,
    is_jobs: Option<bool>,
}
