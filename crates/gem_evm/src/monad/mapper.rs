use std::error::Error;
use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::Zero;
use primitives::StakeType;

use crate::monad::constants::DEFAULT_WITHDRAW_ID;
use crate::monad::contracts::{IMonadStaking, IMonadStakingLens};
use crate::u256::u256_to_biguint;

#[derive(Clone)]
pub struct MonadLensDelegation {
    pub validator_id: u64,
    pub withdraw_id: u8,
    pub state: IMonadStakingLens::DelegationState,
    pub amount: BigUint,
    pub rewards: BigUint,
    pub withdraw_epoch: u64,
    pub completion_timestamp: u64,
}

#[derive(Clone)]
pub struct MonadLensValidatorInfo {
    pub validator_id: u64,
    pub stake: BigUint,
    pub commission: BigUint,
    pub apy_bps: u64,
    pub is_active: bool,
}

#[derive(Clone)]
pub struct MonadLensBalance {
    pub staked: BigUint,
    pub pending: BigUint,
    pub rewards: BigUint,
}

pub fn encode_get_lens_balance(delegator: &str) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IMonadStakingLens::getBalanceCall { delegator }.abi_encode())
}

pub fn decode_get_lens_balance(data: &[u8]) -> Result<MonadLensBalance, Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStakingLens::getBalanceCall::abi_decode_returns(data)?;
    Ok(MonadLensBalance {
        staked: u256_to_biguint(&decoded.staked),
        pending: u256_to_biguint(&decoded.pending),
        rewards: u256_to_biguint(&decoded.rewards),
    })
}

pub fn encode_get_lens_delegations(delegator: &str) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IMonadStakingLens::getDelegationsCall { delegator }.abi_encode())
}

pub fn encode_get_lens_apys(validator_ids: &[u64]) -> Vec<u8> {
    IMonadStakingLens::getAPYsCall {
        validatorIds: validator_ids.to_vec(),
    }
    .abi_encode()
}

pub fn decode_get_lens_apys(data: &[u8]) -> Result<Vec<u64>, Box<dyn Error + Sync + Send>> {
    let (apys_bps,): (Vec<u64>,) = (IMonadStakingLens::getAPYsCall::abi_decode_returns(data)?,);
    Ok(apys_bps)
}

pub fn decode_get_lens_delegations(data: &[u8]) -> Result<Vec<MonadLensDelegation>, Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStakingLens::getDelegationsCall::abi_decode_returns(data)?;

    Ok(decoded
        .into_iter()
        .map(|position| MonadLensDelegation {
            validator_id: position.validatorId,
            withdraw_id: position.withdrawId,
            state: position.state,
            amount: u256_to_biguint(&position.amount),
            rewards: u256_to_biguint(&position.rewards),
            withdraw_epoch: position.withdrawEpoch,
            completion_timestamp: position.completionTimestamp,
        })
        .collect())
}

pub fn encode_get_lens_validators(validator_ids: &[u64]) -> Vec<u8> {
    IMonadStakingLens::getValidatorsCall {
        validatorIds: validator_ids.to_vec(),
    }
    .abi_encode()
}

pub fn decode_get_lens_validators(data: &[u8]) -> Result<(Vec<MonadLensValidatorInfo>, u64), Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStakingLens::getValidatorsCall::abi_decode_returns(data)?;

    Ok((
        decoded
            .validators
            .into_iter()
            .map(|validator| MonadLensValidatorInfo {
                validator_id: validator.validatorId,
                stake: u256_to_biguint(&validator.stake),
                commission: u256_to_biguint(&validator.commission),
                apy_bps: validator.apyBps,
                is_active: validator.isActive,
            })
            .collect(),
        decoded.networkApyBps,
    ))
}

pub fn encode_monad_staking(stake_type: &StakeType, amount: &BigInt) -> Result<(Vec<u8>, BigInt), Box<dyn Error + Sync + Send>> {
    let amount = amount.clone();

    match stake_type {
        StakeType::Stake(validator) => {
            let validator_id = validator.id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            Ok((IMonadStaking::delegateCall { validatorId: validator_id }.abi_encode(), amount))
        }
        StakeType::Unstake(delegation) => {
            let validator_id = delegation.base.validator_id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            let current_withdraw_id = delegation
                .base
                .delegation_id
                .split(':')
                .nth(1)
                .and_then(|id| id.parse::<u8>().ok())
                .unwrap_or(DEFAULT_WITHDRAW_ID);
            let next_withdraw_id = current_withdraw_id.saturating_add(1);
            if amount.sign() == Sign::Minus {
                return Err("Negative values are not supported".into());
            }
            let (_, amount_bytes) = amount.to_bytes_be();
            let amount_u256 = U256::from_be_slice(&amount_bytes);
            Ok((
                IMonadStaking::undelegateCall {
                    validatorId: validator_id,
                    amount: amount_u256,
                    withdrawId: next_withdraw_id,
                }
                .abi_encode(),
                BigInt::zero(),
            ))
        }
        StakeType::Withdraw(delegation) => {
            let validator_id = delegation.base.validator_id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            let withdraw_id = delegation
                .base
                .delegation_id
                .split(':')
                .nth(1)
                .and_then(|id| id.parse::<u8>().ok())
                .ok_or("Invalid withdraw id for Monad")?;

            Ok((
                IMonadStaking::withdrawCall {
                    validatorId: validator_id,
                    withdrawId: withdraw_id,
                }
                .abi_encode(),
                BigInt::zero(),
            ))
        }
        StakeType::Rewards(validators) => {
            let validator = validators.first().ok_or("Missing validator for rewards")?;
            let validator_id = validator.id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            Ok((IMonadStaking::claimRewardsCall { validatorId: validator_id }.abi_encode(), BigInt::zero()))
        }
        _ => Err("Unsupported stake type for Monad".into()),
    }
}
