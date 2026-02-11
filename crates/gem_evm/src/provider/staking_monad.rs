use std::collections::HashMap;
use std::error::Error;

use alloy_primitives::hex;
use chrono::{DateTime, Utc};
use gem_client::Client;
use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator, GrowthProviderType};

use crate::monad::{
    IMonadStakingLens, MONAD_SCALE, MonadLensBalance, MonadLensDelegation, MonadLensValidatorInfo, STAKING_LENS_CONTRACT, decode_get_lens_apys, decode_get_lens_balance,
    decode_get_lens_delegations, decode_get_lens_validators, encode_get_lens_apys, encode_get_lens_balance, encode_get_lens_delegations, encode_get_lens_validators,
};
use crate::rpc::client::EthereumClient;

const MONAD_VALIDATOR_NAMES: &[(u64, &str)] = &[(16, "MonadVision"), (5, "Alchemy"), (10, "Stakin"), (9, "Everstake")];

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_monad_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_apys(&[]);
        let result = self.call_lens(data).await.ok_or_else(|| "Monad staking lens not configured".to_string())??;

        let apys = decode_get_lens_apys(&result)?;
        let apy_bps = apys.into_iter().max().unwrap_or(0);

        if apy_bps == 0 {
            return Ok(None);
        }

        Ok(Some(apy_bps as f64 / 100.0))
    }

    pub async fn get_monad_validators(&self) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        let validator_names: HashMap<u64, &str> = MONAD_VALIDATOR_NAMES.iter().copied().collect();
        let validator_ids = Self::monad_curated_validator_ids();
        let data = encode_get_lens_validators(&validator_ids);
        let result = self.call_lens(data).await.ok_or_else(|| "Monad staking lens not configured".to_string())??;

        let (validators, network_apy_bps) = decode_get_lens_validators(&result)?;
        let network_apy = network_apy_bps as f64 / 100.0;

        Ok(validators
            .into_iter()
            .map(|validator| self.map_lens_validator(&validator, &validator_names, network_apy))
            .collect())
    }

    pub async fn get_monad_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        self.fetch_monad_delegations(address).await
    }

    pub async fn get_monad_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        let balance = self.fetch_monad_balance(address).await?;
        Ok(Some(Self::monad_asset_balance(balance.staked, balance.pending, balance.rewards)))
    }

    async fn fetch_monad_balance(&self, address: &str) -> Result<MonadLensBalance, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_balance(address)?;
        let result = self.call_lens(data).await.ok_or_else(|| "Monad staking lens not configured".to_string())??;

        decode_get_lens_balance(&result)
    }

    async fn fetch_monad_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        let data = encode_get_lens_delegations(address)?;
        let Some(result) = self.call_lens(data).await else {
            return Ok(Vec::new());
        };

        let positions = match result {
            Ok(bytes) => match decode_get_lens_delegations(&bytes) {
                Ok(position_list) => position_list,
                Err(_) => return Ok(Vec::new()),
            },
            Err(_) => return Ok(Vec::new()),
        };

        if positions.is_empty() {
            return Ok(Vec::new());
        }

        let base_delegation_id = address.to_lowercase();
        let mut delegations = Vec::new();

        for position in positions {
            if position.amount.is_zero() {
                continue;
            }

            let state = Self::map_lens_state(&position);
            let completion_date = if position.completion_timestamp == 0 {
                None
            } else {
                DateTime::<Utc>::from_timestamp(position.completion_timestamp as i64, 0)
            };

            delegations.push(DelegationBase {
                asset_id: AssetId::from_chain(Chain::Monad),
                state,
                balance: position.amount,
                shares: BigUint::zero(),
                rewards: position.rewards,
                completion_date,
                delegation_id: format!("{}:{}", base_delegation_id, position.withdraw_id),
                validator_id: position.validator_id.to_string(),
            });
        }

        Ok(delegations)
    }

    fn map_lens_validator(&self, validator: &MonadLensValidatorInfo, validator_names: &HashMap<u64, &str>, network_apy: f64) -> DelegationValidator {
        let validator_name = validator_names
            .get(&validator.validator_id)
            .map(|name| (*name).to_string())
            .unwrap_or_else(|| validator.validator_id.to_string());

        DelegationValidator {
            id: validator.validator_id.to_string(),
            chain: Chain::Monad,
            name: validator_name,
            is_active: validator.is_active,
            commission: Self::lens_commission_rate(&validator.commission),
            apr: if validator.apy_bps > 0 { validator.apy_bps as f64 / 100.0 } else { network_apy },
            provider_type: GrowthProviderType::Stake,
        }
    }

    fn map_lens_state(position: &MonadLensDelegation) -> DelegationState {
        match position.state {
            IMonadStakingLens::DelegationState::Active => DelegationState::Active,
            IMonadStakingLens::DelegationState::Activating => DelegationState::Activating,
            IMonadStakingLens::DelegationState::Deactivating => DelegationState::Deactivating,
            IMonadStakingLens::DelegationState::AwaitingWithdrawal => DelegationState::AwaitingWithdrawal,
            IMonadStakingLens::DelegationState::__Invalid => DelegationState::Inactive,
        }
    }

    async fn call_lens(&self, data: Vec<u8>) -> Option<Result<Vec<u8>, Box<dyn Error + Sync + Send>>> {
        let call = Self::build_lens_call(&data)?;

        Some(
            self.call(call.0, call.1)
                .await
                .map_err(|err| -> Box<dyn Error + Sync + Send> { Box::new(err) })
                .and_then(|result: String| hex::decode(result).map_err(|err| -> Box<dyn Error + Sync + Send> { Box::new(err) })),
        )
    }

    fn build_lens_call(data: &[u8]) -> Option<(String, serde_json::Value)> {
        Some((
            "eth_call".to_string(),
            serde_json::json!([{
                "to": STAKING_LENS_CONTRACT,
                "data": hex::encode_prefixed(data)
            }, "latest"]),
        ))
    }

    fn lens_commission_rate(commission: &BigUint) -> f64 {
        commission.to_f64().unwrap_or(0.0) / MONAD_SCALE
    }

    fn monad_asset_balance(staked: BigUint, pending: BigUint, rewards: BigUint) -> AssetBalance {
        AssetBalance::new_balance(AssetId::from_chain(Chain::Monad), primitives::Balance::stake_balance(staked, pending, Some(rewards)))
    }

    fn monad_curated_validator_ids() -> Vec<u64> {
        MONAD_VALIDATOR_NAMES.iter().map(|(id, _)| *id).collect()
    }
}
