use num_bigint::BigUint;
use serde::Deserialize;
use serde_serializers::deserialize_f64_from_str;

use super::contracts::WithdrawRequest;

#[derive(Debug, Deserialize)]
pub struct QueueStatsResponse {
    pub validator_activation_time: u64,
    pub validator_exit_time: u64,
    pub validator_withdraw_time: u64,
    pub validator_adding_delay: u64,
}

#[derive(Debug, Deserialize)]
pub struct StatsResponse {
    #[serde(deserialize_with = "deserialize_f64_from_str")]
    pub apr: f64,
}

#[derive(Debug)]
pub struct AccountState {
    pub deposited_balance: BigUint,
    pub pending_balance: BigUint,
    pub pending_deposited_balance: BigUint,
    pub withdraw_request: WithdrawRequest,
    pub restaked_reward: BigUint,
}
