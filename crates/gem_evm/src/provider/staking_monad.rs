use std::collections::HashMap;
use std::error::Error;

use alloy_primitives::hex;
use chrono::{DateTime, Duration, Utc};
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

use crate::constants::STAKING_VALIDATORS_LIMIT;
use crate::monad::{
    ACTIVE_VALIDATOR_SET, DEFAULT_WITHDRAW_ID, MONAD_BLOCK_REWARD_MON, MONAD_BLOCK_TIME_SECONDS, MONAD_BLOCKS_PER_YEAR, MONAD_BOUNDARY_BLOCK_PERIOD,
    MONAD_MAX_WITHDRAW_IDS, MONAD_SCALE, MonadDelegatorState, MonadValidator, MonadWithdrawalRequest, STAKING_CONTRACT, decode_get_delegations,
    decode_get_delegator, decode_get_epoch, decode_get_validator, decode_get_withdrawal_request, encode_get_delegations, encode_get_delegator,
    encode_get_epoch, encode_get_validator, encode_get_withdrawal_request,
};
use crate::rpc::client::EthereumClient;

const MONAD_VALIDATOR_NAMES: &[(u64, &str)] = &[
    (16, "MonadVision"),
    (3, "gmonads.com"),
    (6, "ProStaking"),
    (123, "P2P.org"),
    (5, "Alchemy"),
    (128, "Laine"),
    (10, "Stakin"),
    (4, "B-Harvest"),
    (7, "Staking4All"),
    (72, "01node"),
    (43, "P-OPS Team"),
    (92, "CMS Holdings"),
    (9, "Everstake"),
    (23, "Nansen | Stake to Stack Points"),
    (11, "Pier Two"),
    (12, "Chorus One"),
    (78, "Validation Cloud"),
    (114, "nadradar"),
    (17, "Cosmostation"),
    (75, "laminatedlabs"),
];

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_monad_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.fetch_monad_validators(&Self::monad_all_validator_ids()).await?;
        let total_stake = Self::total_validator_stake(&validators);
        Self::calculate_monad_network_apy(&total_stake)
    }

    pub async fn get_monad_validators(&self, fallback_apy: f64) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let total_stake = Self::total_validator_stake(&self.fetch_monad_validators(&Self::monad_all_validator_ids()).await?);
        let validators = self.fetch_monad_validators(&Self::monad_featured_validator_ids()).await?;
        let validator_names = Self::monad_validator_names();
        let network_apy = Self::calculate_monad_network_apy(&total_stake)?;

        validators
            .into_iter()
            .map(|(id, val)| -> Result<DelegationValidator, Box<dyn Error + Sync + Send>> {
                let validator_apy = Self::calculate_validator_apy(&val, &total_stake)?.or(network_apy).unwrap_or(fallback_apy);
                let name_override = validator_names.get(&id).map(|name| name.to_string());
                Ok(self.map_validator(id, &val, validator_apy, name_override))
            })
            .collect()
    }

    pub async fn get_monad_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let mut validator_ids = Vec::new();
        let mut start_val_id: u64 = 0;

        loop {
            let data = encode_get_delegations(address, start_val_id)?;
            let result = self.call_staking(data).await?;
            let page = decode_get_delegations(&result)?;

            validator_ids.extend_from_slice(&page.validator_ids);

            if page.is_done || validator_ids.len() as u16 >= STAKING_VALIDATORS_LIMIT {
                break;
            }

            start_val_id = page.next;
        }

        let mut delegations = Vec::new();
        let current_epoch = self.fetch_monad_epoch().await?;
        let delegator_states = self.fetch_monad_delegator_states(address, &validator_ids).await?;
        let withdrawal_requests = self.fetch_monad_withdrawal_requests(address, &validator_ids).await?;

        for (validator_id, delegator_state) in delegator_states {
            let withdrawals = withdrawal_requests.get(&validator_id);
            self.push_delegations(address, validator_id, &delegator_state, withdrawals, current_epoch, &mut delegations);
        }

        Ok(delegations)
    }

    pub async fn get_monad_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let delegations = self.get_monad_delegations(address).await?;

        let mut staked = BigUint::zero();
        let mut pending = BigUint::zero();
        let mut rewards = BigUint::zero();

        for delegation in &delegations {
            match delegation.state {
                DelegationState::Active => {
                    staked += &delegation.balance;
                    rewards += &delegation.rewards;
                }
                DelegationState::Activating | DelegationState::Deactivating | DelegationState::AwaitingWithdrawal | DelegationState::Pending => {
                    pending += &delegation.balance;
                }
                DelegationState::Inactive => {}
            }
        }

        Ok(Some(AssetBalance::new_balance(
            AssetId::from_chain(Chain::Monad),
            primitives::Balance::stake_balance(staked, pending, Some(rewards)),
        )))
    }

    async fn fetch_monad_validators(&self, validator_ids: &[u64]) -> Result<Vec<(u64, MonadValidator)>, Box<dyn Error + Sync + Send>> {
        if validator_ids.is_empty() {
            return Ok(vec![]);
        }

        let calls = validator_ids
            .iter()
            .map(|validator_id| {
                let data = encode_get_validator(*validator_id);
                Self::build_staking_call(&data)
            })
            .collect::<Vec<_>>();

        let results: Vec<String> = self.client.batch_call::<String>(calls).await?.extract();

        let mut validators = Vec::new();
        for (idx, result) in results.into_iter().enumerate() {
            if let Some(validator_id) = validator_ids.get(idx) {
                let decoded_bytes = hex::decode(result)?;
                let validator = decode_get_validator(&decoded_bytes)?;
                validators.push((*validator_id, validator));
            }
        }

        Ok(validators)
    }

    async fn fetch_monad_delegator_states(
        &self,
        address: &str,
        validator_ids: &[u64],
    ) -> Result<Vec<(u64, MonadDelegatorState)>, Box<dyn Error + Sync + Send>> {
        if validator_ids.is_empty() {
            return Ok(vec![]);
        }

        let calls = validator_ids
            .iter()
            .map(|validator_id| {
                let data = encode_get_delegator(*validator_id, address).unwrap_or_default();
                Self::build_staking_call(&data)
            })
            .collect::<Vec<_>>();

        let results: Vec<String> = self.client.batch_call::<String>(calls).await?.extract();
        let mut states = Vec::new();

        for (idx, result) in results.into_iter().enumerate() {
            if let Some(validator_id) = validator_ids.get(idx) {
                let decoded_bytes = hex::decode(result)?;
                let state = decode_get_delegator(&decoded_bytes)?;
                states.push((*validator_id, state));
            }
        }

        Ok(states)
    }

    async fn fetch_monad_withdrawal_requests(
        &self,
        address: &str,
        validator_ids: &[u64],
    ) -> Result<HashMap<u64, Vec<MonadWithdrawalRequest>>, Box<dyn Error + Sync + Send>> {
        if validator_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut active_validator_ids = validator_ids.to_vec();
        let mut withdrawals: HashMap<u64, Vec<MonadWithdrawalRequest>> = HashMap::new();

        for withdraw_id in 0..MONAD_MAX_WITHDRAW_IDS {
            if active_validator_ids.is_empty() {
                break;
            }

            let calls = active_validator_ids
                .iter()
                .map(|validator_id| {
                    let data = encode_get_withdrawal_request(*validator_id, address, withdraw_id).unwrap_or_default();
                    Self::build_staking_call(&data)
                })
                .collect::<Vec<_>>();

            let results: Vec<String> = self.client.batch_call::<String>(calls).await?.extract();
            let mut next_round_ids = Vec::new();

            for (idx, result) in results.into_iter().enumerate() {
                if let Some(validator_id) = active_validator_ids.get(idx) {
                    let decoded_bytes = hex::decode(result)?;
                    let mut request = decode_get_withdrawal_request(&decoded_bytes)?;
                    request.withdraw_id = withdraw_id;

                    if request.amount.is_zero() {
                        continue;
                    }

                    withdrawals.entry(*validator_id).or_default().push(request);
                    next_round_ids.push(*validator_id);
                }
            }

            active_validator_ids = next_round_ids;
        }

        Ok(withdrawals)
    }

    async fn fetch_monad_epoch(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        let data = encode_get_epoch();
        let result = self.call_staking(data).await?;
        let (epoch, _) = decode_get_epoch(&result)?;
        Ok(epoch)
    }

    fn map_validator(&self, validator_id: u64, validator: &MonadValidator, apy: f64, name_override: Option<String>) -> DelegationValidator {
        let validator_name = name_override.unwrap_or_else(|| validator_id.to_string());
        let is_active = validator.flags == 0 && !validator.stake.is_zero();

        DelegationValidator {
            id: validator_id.to_string(),
            chain: Chain::Monad,
            name: validator_name,
            is_active,
            commission: validator.commission_rate(),
            apr: apy,
        }
    }

    fn push_delegations(
        &self,
        address: &str,
        validator_id: u64,
        delegator_state: &MonadDelegatorState,
        withdrawals: Option<&Vec<MonadWithdrawalRequest>>,
        current_epoch: u64,
        delegations: &mut Vec<DelegationBase>,
    ) {
        let pending_balance = &delegator_state.delta_stake + &delegator_state.next_delta_stake;
        let current_withdraw_id = withdrawals
            .and_then(|reqs| reqs.iter().map(|r| r.withdraw_id).max())
            .unwrap_or(DEFAULT_WITHDRAW_ID);
        let base_delegation_id = address.to_lowercase();
        let delegation_id = format!("{}:{}", base_delegation_id, current_withdraw_id);

        if !delegator_state.stake.is_zero() {
            delegations.push(DelegationBase {
                asset_id: AssetId::from_chain(Chain::Monad),
                state: DelegationState::Active,
                balance: delegator_state.stake.clone(),
                shares: BigUint::zero(),
                rewards: delegator_state.unclaimed_rewards.clone(),
                completion_date: None,
                delegation_id: delegation_id.clone(),
                validator_id: validator_id.to_string(),
            });
        }

        if !pending_balance.is_zero() {
            delegations.push(DelegationBase {
                asset_id: AssetId::from_chain(Chain::Monad),
                state: DelegationState::Activating,
                balance: pending_balance,
                shares: BigUint::zero(),
                rewards: BigUint::zero(),
                completion_date: None,
                delegation_id: delegation_id.clone(),
                validator_id: validator_id.to_string(),
            });
        }

        if let Some(requests) = withdrawals {
            for request in requests {
                if request.amount.is_zero() {
                    continue;
                }

                let is_ready = request.withdraw_epoch < current_epoch;
                let completion_date = Self::withdrawal_completion_date(request.withdraw_epoch, current_epoch);
                let state = if is_ready { DelegationState::AwaitingWithdrawal } else { DelegationState::Deactivating };

                let withdrawal_delegation_id = format!("{}:{}", base_delegation_id, request.withdraw_id);

                delegations.push(DelegationBase {
                    asset_id: AssetId::from_chain(Chain::Monad),
                    state,
                    balance: request.amount.clone(),
                    shares: BigUint::zero(),
                    rewards: BigUint::zero(),
                    completion_date,
                    delegation_id: withdrawal_delegation_id,
                    validator_id: validator_id.to_string(),
                });
            }
        }
    }

    async fn call_staking(&self, data: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
        let call = Self::build_staking_call(&data);
        let result: String = self.call(call.0, call.1).await?;
        Ok(hex::decode(result)?)
    }

    fn build_staking_call(data: &[u8]) -> (String, serde_json::Value) {
        (
            "eth_call".to_string(),
            serde_json::json!([{
                "to": STAKING_CONTRACT,
                "data": hex::encode_prefixed(data)
            }, "latest"]),
        )
    }

    fn total_validator_stake(validators: &[(u64, MonadValidator)]) -> BigUint {
        validators.iter().fold(BigUint::zero(), |acc, (_, validator)| acc + &validator.stake)
    }

    fn calculate_monad_network_apy(total_stake: &BigUint) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let total_stake_mon = Self::stake_to_mon(total_stake)?;
        if total_stake_mon == 0.0 {
            return Ok(None);
        }

        let annual_rewards = MONAD_BLOCK_REWARD_MON * MONAD_BLOCKS_PER_YEAR;
        Ok(Some((annual_rewards / total_stake_mon) * 100.0))
    }

    fn calculate_validator_apy(validator: &MonadValidator, total_stake: &BigUint) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let total_stake_mon = Self::stake_to_mon(total_stake)?;
        let validator_stake_mon = validator.stake_in_mon().unwrap_or(0.0);

        if total_stake_mon == 0.0 || validator_stake_mon == 0.0 {
            return Ok(None);
        }

        let stake_weight = validator_stake_mon / total_stake_mon;
        let expected_blocks = stake_weight * MONAD_BLOCKS_PER_YEAR;
        let gross_rewards = expected_blocks * MONAD_BLOCK_REWARD_MON;

        let commission = validator.commission_rate().clamp(0.0, 1.0);
        let net_rewards = gross_rewards * (1.0 - commission);

        Ok(Some((net_rewards / validator_stake_mon) * 100.0))
    }

    fn stake_to_mon(stake: &BigUint) -> Result<f64, Box<dyn Error + Sync + Send>> {
        let stake_value = stake
            .to_f64()
            .ok_or_else(|| "Failed to convert Monad stake to floating point value".to_string())?;
        Ok(stake_value / MONAD_SCALE)
    }

    fn monad_all_validator_ids() -> Vec<u64> {
        (0..ACTIVE_VALIDATOR_SET).map(u64::from).collect()
    }

    fn monad_featured_validator_ids() -> Vec<u64> {
        MONAD_VALIDATOR_NAMES.iter().map(|(id, _)| *id).collect()
    }

    fn monad_validator_names() -> HashMap<u64, &'static str> {
        MONAD_VALIDATOR_NAMES.iter().copied().collect()
    }

    fn withdrawal_completion_date(withdraw_epoch: u64, current_epoch: u64) -> Option<DateTime<Utc>> {
        if withdraw_epoch < current_epoch {
            return None;
        }

        let epoch_seconds = (MONAD_BOUNDARY_BLOCK_PERIOD as f64 * MONAD_BLOCK_TIME_SECONDS) as i64;
        if epoch_seconds <= 0 {
            return None;
        }

        let remaining_epochs = withdraw_epoch.saturating_sub(current_epoch).saturating_add(1) as i64;
        Some(Utc::now() + Duration::seconds(epoch_seconds.saturating_mul(remaining_epochs)))
    }
}
