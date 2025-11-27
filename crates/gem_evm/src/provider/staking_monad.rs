use crate::address::ethereum_address_checksum;
use crate::constants::STAKING_VALIDATORS_LIMIT;
use crate::monad::{
    ACTIVE_VALIDATOR_SET, MONAD_BLOCK_REWARD_MON, MONAD_BLOCKS_PER_YEAR, MONAD_WEI_PER_MON, MonadDelegatorState, MonadValidator, STAKING_CONTRACT,
    decode_get_delegations, decode_get_delegator, decode_get_validator, decode_get_validator_set, encode_get_delegations, encode_get_delegator,
    encode_get_validator, encode_get_validator_set,
};
use crate::rpc::client::EthereumClient;
use alloy_primitives::hex;
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use num_traits::Zero;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};
use std::error::Error;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_monad_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.fetch_monad_validator_set().await?;
        let total_stake = Self::total_validator_stake(&validators);
        Self::calculate_monad_network_apy(&total_stake)
    }

    pub async fn get_monad_validators(&self, fallback_apy: f64) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validators = self.fetch_monad_validator_set().await?;
        let total_stake = Self::total_validator_stake(&validators);
        let network_apy = Self::calculate_monad_network_apy(&total_stake)?;

        validators
            .into_iter()
            .map(|(_, val)| -> Result<DelegationValidator, Box<dyn Error + Sync + Send>> {
                let validator_apy = Self::calculate_validator_apy(&val, &total_stake)?.or(network_apy).unwrap_or(fallback_apy);
                Ok(self.map_validator(&val, validator_apy))
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
        let delegator_states = self.fetch_monad_delegator_states(address, &validator_ids).await?;
        for (validator_id, delegator_state) in delegator_states {
            self.push_delegations(address, validator_id, &delegator_state, &mut delegations);
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

    async fn fetch_monad_validator_set(&self) -> Result<Vec<(u64, MonadValidator)>, Box<dyn Error + Sync + Send>> {
        let validator_ids = self.fetch_monad_validator_ids().await?;
        self.fetch_monad_validators(&validator_ids).await
    }

    async fn fetch_monad_validator_ids(&self) -> Result<Vec<u64>, Box<dyn Error + Sync + Send>> {
        let mut ids = Vec::new();
        let mut start_index: u32 = 0;

        loop {
            let data = encode_get_validator_set(start_index);
            let result = self.call_staking(data).await?;
            let page = decode_get_validator_set(&result)?;

            ids.extend_from_slice(&page.validator_ids);
            if page.is_done || ids.len() as u32 >= ACTIVE_VALIDATOR_SET {
                break;
            }

            start_index = page.next;
        }

        Ok(ids)
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

    fn map_validator(&self, validator: &MonadValidator, apy: f64) -> DelegationValidator {
        let auth_address = ethereum_address_checksum(&validator.auth_address.to_string()).unwrap_or_else(|_| validator.auth_address.to_string());
        let is_active = validator.flags == 0;

        DelegationValidator {
            id: auth_address.clone(),
            chain: Chain::Monad,
            name: auth_address,
            is_active,
            commission: validator.commission_rate(),
            apr: apy,
        }
    }

    fn push_delegations(&self, address: &str, validator_id: u64, delegator_state: &MonadDelegatorState, delegations: &mut Vec<DelegationBase>) {
        let pending_balance = &delegator_state.delta_stake + &delegator_state.next_delta_stake;
        let state = if !delegator_state.stake.is_zero() {
            DelegationState::Active
        } else if !pending_balance.is_zero() {
            DelegationState::Activating
        } else {
            DelegationState::Pending
        };

        let delegation_id = ethereum_address_checksum(address).unwrap_or_else(|_| address.to_string());

        delegations.push(DelegationBase {
            asset_id: AssetId::from_chain(Chain::Monad),
            state,
            balance: delegator_state.stake.clone(),
            shares: BigUint::zero(),
            rewards: delegator_state.unclaimed_rewards.clone(),
            completion_date: None,
            delegation_id: delegation_id.clone(),
            validator_id: validator_id.to_string(),
        });

        if !pending_balance.is_zero() {
            delegations.push(DelegationBase {
                asset_id: AssetId::from_chain(Chain::Monad),
                state: DelegationState::Activating,
                balance: pending_balance,
                shares: BigUint::zero(),
                rewards: BigUint::zero(),
                completion_date: None,
                delegation_id,
                validator_id: validator_id.to_string(),
            });
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
        Ok(stake_value / MONAD_WEI_PER_MON)
    }
}
