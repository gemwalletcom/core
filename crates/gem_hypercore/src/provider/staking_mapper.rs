use crate::models::balance::{DelegationBalance, StakeBalance, Validator};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, Chain, DelegationBase, DelegationState, DelegationValidator};

pub fn map_staking_validators(validators: Vec<Validator>, chain: Chain, apy: Option<f64>) -> Vec<DelegationValidator> {
    let calculated_apy = apy.unwrap_or_else(|| Validator::max_apr(validators.clone()));
    let mut result: Vec<DelegationValidator> = validators
        .into_iter()
        .map(|x| DelegationValidator::stake(chain, x.validator_address(), x.name, x.is_active, x.commission, calculated_apy))
        .collect();

    result.push(DelegationValidator::system(chain));

    result
}

pub fn map_staking_delegations(delegations: Vec<DelegationBalance>, stake_balance: StakeBalance, chain: Chain) -> Vec<DelegationBase> {
    let native_decimals = Asset::from_chain(chain).decimals as u32;
    let mut result: Vec<DelegationBase> = delegations
        .into_iter()
        .map(|x| DelegationBase {
            asset_id: chain.as_asset_id(),
            state: DelegationState::Active,
            balance: BigNumberFormatter::value_from_amount_biguint(&x.amount, native_decimals).unwrap_or_default(),
            shares: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            completion_date: None,
            delegation_id: x.validator_address(),
            validator_id: x.validator_address(),
        })
        .collect();

    let pending = BigNumberFormatter::value_from_amount_biguint(&stake_balance.total_pending_withdrawal, native_decimals).unwrap_or_default();
    if pending > BigUint::from(0u32) {
        result.push(DelegationBase {
            asset_id: chain.as_asset_id(),
            state: DelegationState::Pending,
            balance: pending,
            shares: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            completion_date: None,
            delegation_id: DelegationValidator::SYSTEM_ID.to_string(),
            validator_id: DelegationValidator::SYSTEM_ID.to_string(),
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::balance::ValidatorStats;
    use primitives::{Chain, DelegationState};

    fn stake_balance(total_pending_withdrawal: &str) -> StakeBalance {
        StakeBalance {
            delegated: "0".to_string(),
            undelegated: "0".to_string(),
            total_pending_withdrawal: total_pending_withdrawal.to_string(),
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
        assert_eq!(result[0].apr, 15.0);

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
        assert_eq!(result[0].apr, 10.0);
        assert_eq!(result[1].id, DelegationValidator::SYSTEM_ID);
    }

    #[test]
    fn test_map_staking_delegations() {
        let delegations: Vec<DelegationBalance> = serde_json::from_str(include_str!("../../testdata/staking_delegations.json")).unwrap();

        let result = map_staking_delegations(delegations, stake_balance("0"), Chain::HyperCore);

        assert_eq!(result.len(), 2);

        let delegation1 = &result[0];
        assert_eq!(delegation1.asset_id.chain, Chain::HyperCore);
        assert_eq!(delegation1.validator_id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(delegation1.delegation_id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(delegation1.balance.to_string(), "271936493373");
        assert_eq!(delegation1.state, DelegationState::Active);
        assert_eq!(delegation1.shares, num_bigint::BigUint::from(0u32));
        assert_eq!(delegation1.rewards, num_bigint::BigUint::from(0u32));
        assert!(delegation1.completion_date.is_none());

        let delegation2 = &result[1];
        assert_eq!(delegation2.validator_id, "0xaBCDefF4b3727B83A23697500EEf089020DF2cD2");
        assert_eq!(delegation2.balance.to_string(), "1814578086");
    }

    #[test]
    fn test_map_staking_delegations_pending_withdrawal() {
        let result = map_staking_delegations(vec![], stake_balance("0.015"), Chain::HyperCore);

        assert_eq!(result.len(), 1);
        let pending = &result[0];
        assert_eq!(pending.state, DelegationState::Pending);
        assert_eq!(pending.validator_id, DelegationValidator::SYSTEM_ID);
        assert_eq!(pending.balance.to_string(), "1500000");
        assert!(pending.completion_date.is_none());
    }

    #[test]
    fn test_map_staking_delegations_no_pending_withdrawal() {
        let result = map_staking_delegations(vec![], stake_balance("0"), Chain::HyperCore);

        assert!(result.is_empty());
    }
}
