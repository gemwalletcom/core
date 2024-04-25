#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiPay {
    tx_bytes: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiPayRequest {
    sender_address: String,
    recipient_address: String,
    coins: Vec<String>,
    gas: Option<String>,
    amount: String,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiAddStakeRequest {
    sender_address: String,
    validator_address: String,
    coins: Vec<String>,
    amount: String,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiUnstakeRequest {
    sender_address: String,
    delegation_id: String,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiMoveCallRequest {
    sender_address: String,
    object_id: String,
    module: String,
    function: String,
    arguments: Vec<String>,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiSplitCoinRequest {
    sender_address: String,
    coin: String,
    split_amounts: Vec<String>,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiBatchRequest {
    sender_address: String,
    gas_budget: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiBroadcastTransaction {
    digest: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiStakeDelegation {
    validator_address: String,
    staking_pool: String,
    stakes: Vec<SuiStake>,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiSystemState {
    epoch: String,
    epoch_start_timestamp_ms: String,
    epoch_duration_ms: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiStake {
    staked_sui_id: String,
    status: String,
    principal: String,
    stake_request_epoch: String,
    stake_active_epoch: String,
    estimated_reward: Option<String>,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiValidators {
    apys: Vec<SuiValidator>,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiValidator {
    address: String,
    apy: f64,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiCoinMetadata {
    decimals: i32,
    name: String,
    symbol: String,
}
