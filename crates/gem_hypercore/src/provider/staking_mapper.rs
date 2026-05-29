use crate::models::balance::{DelegationBalance, DelegatorHistoryEntry, Validator, WithdrawalPhase};
use chrono::{DateTime, Duration, Utc};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, Chain, DelegationBase, DelegationState, DelegationValidator};
use std::str::FromStr;

pub fn map_staking_validators(validators: Vec<Validator>, chain: Chain, apy: Option<f64>) -> Vec<DelegationValidator> {
    let calculated_apy = apy.unwrap_or_else(|| Validator::max_apr(validators.clone()));
    let mut result: Vec<DelegationValidator> = validators
        .into_iter()
        .map(|x| DelegationValidator::stake(chain, x.validator_address(), x.name, x.is_active, x.commission, calculated_apy))
        .collect();

    result.push(DelegationValidator::system(chain));

    result
}

pub fn map_staking_delegations(delegations: Vec<DelegationBalance>, history: Vec<DelegatorHistoryEntry>, now: DateTime<Utc>, chain: Chain) -> Vec<DelegationBase> {
    let native_decimals = Asset::from_chain(chain).decimals as u32;
    let mut result: Vec<DelegationBase> = delegations
        .into_iter()
        .map(|x| DelegationBase {
            asset_id: chain.as_asset_id(),
            state: DelegationState::Active,
            balance: parse_balance(&x.amount.to_string(), native_decimals),
            shares: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            completion_date: None,
            delegation_id: x.validator_address(),
            validator_id: x.validator_address(),
        })
        .collect();

    result.extend(map_pending_withdrawals(history, now, chain, native_decimals));
    result
}

fn map_pending_withdrawals(history: Vec<DelegatorHistoryEntry>, now: DateTime<Utc>, chain: Chain, native_decimals: u32) -> Vec<DelegationBase> {
    let lock_time = chain.config().stake.as_ref().map(|stake| stake.lock_time).unwrap_or_default();
    let lock = Duration::seconds(lock_time as i64);

    history
        .into_iter()
        .filter_map(|entry| {
            let withdrawal = entry.delta.withdrawal?;
            if withdrawal.phase != WithdrawalPhase::Initiated {
                return None;
            }
            let completion_date = DateTime::from_timestamp_millis(entry.time as i64)? + lock;
            if completion_date <= now {
                return None;
            }
            Some(DelegationBase {
                asset_id: chain.as_asset_id(),
                state: DelegationState::Pending,
                balance: parse_balance(&withdrawal.amount.to_string(), native_decimals),
                shares: BigUint::from(0u32),
                rewards: BigUint::from(0u32),
                completion_date: Some(completion_date),
                delegation_id: format!("unstaking_{}", entry.time),
                validator_id: DelegationValidator::SYSTEM_ID.to_string(),
            })
        })
        .collect()
}

fn parse_balance(amount: &str, native_decimals: u32) -> BigUint {
    BigNumberFormatter::value_from_amount(amount, native_decimals)
        .ok()
        .and_then(|s| BigUint::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::balance::{DelegatorDelta, DelegatorWithdrawal, ValidatorStats};
    use primitives::{Chain, DelegationState};

    fn withdrawal_entry(time: u64, amount: f64, phase: WithdrawalPhase) -> DelegatorHistoryEntry {
        DelegatorHistoryEntry {
            time,
            delta: DelegatorDelta {
                withdrawal: Some(DelegatorWithdrawal { amount, phase }),
            },
        }
    }

    #[test]
    fn test_map_staking_validators() {
        let validators = vec![Validator {
            validator: "0x5ac99df645f3414876c816caa18b2d234024b487".to_string(),
            name: "Test Validator".to_string(),
            commission: 5.0,
            is_active: true,
            stats: vec![("test".to_string(), ValidatorStats { predicted_apr: 0.15 })],
        }];

        let result = map_staking_validators(validators, Chain::HyperCore, None);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Test Validator");
        assert_eq!(result[0].id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(result[0].chain, Chain::HyperCore);
        assert!(result[0].is_active);
        assert_eq!(result[0].commission, 5.0);
        assert_eq!(result[0].apr, 15.0); // max_apr * 100

        let system = &result[1];
        assert_eq!(system.id, DelegationValidator::SYSTEM_ID);
        assert_eq!(system.name, DelegationValidator::SYSTEM_NAME);
        assert!(system.is_active);
    }

    #[test]
    fn test_map_staking_validators_with_apy() {
        let validators = vec![Validator {
            validator: "0x5ac99df645f3414876c816caa18b2d234024b487".to_string(),
            name: "Test Validator".to_string(),
            commission: 5.0,
            is_active: true,
            stats: vec![],
        }];

        let result = map_staking_validators(validators, Chain::HyperCore, Some(10.0));
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].apr, 10.0); // Uses provided APY
        assert_eq!(result[1].id, DelegationValidator::SYSTEM_ID);
    }

    #[test]
    fn test_map_staking_delegations() {
        let delegations: Vec<DelegationBalance> = serde_json::from_str(include_str!("../../testdata/staking_delegations.json")).unwrap();

        let result = map_staking_delegations(delegations, vec![], Utc::now(), Chain::HyperCore);

        assert_eq!(result.len(), 2);

        let delegation1 = &result[0];
        assert_eq!(delegation1.asset_id.chain, Chain::HyperCore);
        assert_eq!(delegation1.validator_id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(delegation1.delegation_id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(delegation1.balance.to_string(), "271936493373");
        assert!(matches!(delegation1.state, DelegationState::Active));
        assert_eq!(delegation1.shares, num_bigint::BigUint::from(0u32));
        assert_eq!(delegation1.rewards, num_bigint::BigUint::from(0u32));
        assert!(delegation1.completion_date.is_none());

        let delegation2 = &result[1];
        assert_eq!(delegation2.validator_id, "0xaBCDefF4b3727B83A23697500EEf089020DF2cD2");
        assert_eq!(delegation2.balance.to_string(), "1814578086");
    }

    #[test]
    fn test_map_staking_delegations_pending_withdrawals() {
        let now = DateTime::from_timestamp(1_780_000_000, 0).unwrap();
        let history = vec![
            withdrawal_entry(1_779_913_600_000, 1.5, WithdrawalPhase::Initiated),
            withdrawal_entry(1_779_308_800_000, 2.0, WithdrawalPhase::Initiated),
            withdrawal_entry(1_779_950_000_000, 3.0, WithdrawalPhase::Finalized),
            DelegatorHistoryEntry {
                time: 1_779_950_000_001,
                delta: DelegatorDelta { withdrawal: None },
            },
        ];

        let result = map_staking_delegations(vec![], history, now, Chain::HyperCore);

        assert_eq!(result.len(), 1);
        let pending = &result[0];
        assert!(matches!(pending.state, DelegationState::Pending));
        assert_eq!(pending.validator_id, DelegationValidator::SYSTEM_ID);
        assert_eq!(pending.balance.to_string(), "150000000");
        assert_eq!(pending.delegation_id, "unstaking_1779913600000");
        assert_eq!(pending.completion_date, Some(now + Duration::seconds(518_400)));
        assert!(pending.completion_date.unwrap() > now);
    }
}
