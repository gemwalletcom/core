use crate::monad::constants::{DEFAULT_WITHDRAW_ID, MONAD_COMMISSION_SCALE, MONAD_WEI_PER_MON};
use crate::monad::contracts::IMonadStaking;
use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{ToPrimitive, Zero};
use primitives::StakeType;
use std::error::Error;
use std::str::FromStr;

pub struct MonadValidator {
    pub auth_address: Address,
    pub flags: u64,
    pub stake: BigUint,
    pub commission: BigUint,
    pub unclaimed_rewards: BigUint,
}

pub struct MonadDelegatorState {
    pub stake: BigUint,
    pub delta_stake: BigUint,
    pub next_delta_stake: BigUint,
    pub unclaimed_rewards: BigUint,
}

pub struct MonadIdsPage<T> {
    pub is_done: bool,
    pub next: T,
    pub validator_ids: Vec<u64>,
}

pub type MonadDelegationsPage = MonadIdsPage<u64>;
pub type MonadValidatorSetPage = MonadIdsPage<u32>;

pub fn encode_get_validator_set(start_index: u32) -> Vec<u8> {
    IMonadStaking::getConsensusValidatorSetCall { startIndex: start_index }.abi_encode()
}

pub fn decode_get_validator_set(data: &[u8]) -> Result<MonadValidatorSetPage, Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStaking::getConsensusValidatorSetCall::abi_decode_returns(data)?;
    Ok(MonadValidatorSetPage {
        is_done: decoded.isDone,
        next: decoded.nextIndex,
        validator_ids: decoded.valIds,
    })
}

pub fn encode_get_delegations(delegator: &str, start_val_id: u64) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IMonadStaking::getDelegationsCall {
        delegator,
        startValId: start_val_id,
    }
    .abi_encode())
}

pub fn decode_get_delegations(data: &[u8]) -> Result<MonadDelegationsPage, Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStaking::getDelegationsCall::abi_decode_returns(data)?;
    Ok(MonadDelegationsPage {
        is_done: decoded.isDone,
        next: decoded.nextValId,
        validator_ids: decoded.valIds,
    })
}

pub fn encode_get_delegator(validator_id: u64, delegator: &str) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IMonadStaking::getDelegatorCall {
        validatorId: validator_id,
        delegator,
    }
    .abi_encode())
}

pub fn decode_get_delegator(data: &[u8]) -> Result<MonadDelegatorState, Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStaking::getDelegatorCall::abi_decode_returns(data)?;

    Ok(MonadDelegatorState {
        stake: BigUint::from_bytes_be(&decoded.stake.to_be_bytes::<32>()),
        delta_stake: BigUint::from_bytes_be(&decoded.deltaStake.to_be_bytes::<32>()),
        next_delta_stake: BigUint::from_bytes_be(&decoded.nextDeltaStake.to_be_bytes::<32>()),
        unclaimed_rewards: BigUint::from_bytes_be(&decoded.unclaimedRewards.to_be_bytes::<32>()),
    })
}

pub fn encode_get_validator(validator_id: u64) -> Vec<u8> {
    IMonadStaking::getValidatorCall { validatorId: validator_id }.abi_encode()
}

pub fn decode_get_validator(data: &[u8]) -> Result<MonadValidator, Box<dyn Error + Sync + Send>> {
    let decoded = IMonadStaking::getValidatorCall::abi_decode_returns(data)?;

    Ok(MonadValidator {
        auth_address: decoded.authAddress,
        flags: decoded.flags,
        stake: BigUint::from_bytes_be(&decoded.stake.to_be_bytes::<32>()),
        commission: BigUint::from_bytes_be(&decoded.commission.to_be_bytes::<32>()),
        unclaimed_rewards: BigUint::from_bytes_be(&decoded.unclaimedRewards.to_be_bytes::<32>()),
    })
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
            if amount.sign() == Sign::Minus {
                return Err("Negative values are not supported".into());
            }
            let (_, amount_bytes) = amount.to_bytes_be();
            let amount_u256 = U256::from_be_slice(&amount_bytes);
            Ok((
                IMonadStaking::undelegateCall {
                    validatorId: validator_id,
                    amount: amount_u256,
                    withdrawId: DEFAULT_WITHDRAW_ID,
                }
                .abi_encode(),
                BigInt::zero(),
            ))
        }
        StakeType::Withdraw(delegation) => {
            let validator_id = delegation.base.validator_id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            Ok((
                IMonadStaking::withdrawCall {
                    validatorId: validator_id,
                    withdrawId: DEFAULT_WITHDRAW_ID,
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

impl MonadValidator {
    pub fn commission_rate(&self) -> f64 {
        self.commission.to_f64().unwrap_or(0.0) / MONAD_COMMISSION_SCALE
    }

    pub fn stake_in_mon(&self) -> Option<f64> {
        self.stake.to_f64().map(|value| value / MONAD_WEI_PER_MON)
    }
}
