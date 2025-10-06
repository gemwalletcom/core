use crate::models::balance::{DelegationBalance, Validator};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};
use std::str::FromStr;

pub fn map_staking_validators(validators: Vec<Validator>, chain: Chain, apy: Option<f64>) -> Vec<DelegationValidator> {
    let calculated_apy = apy.unwrap_or_else(|| Validator::max_apr(validators.clone()));
    validators
        .into_iter()
        .map(|x| DelegationValidator {
            chain,
            id: x.validator_address(),
            name: x.name,
            is_active: x.is_active,
            commission: x.commission,
            apr: calculated_apy,
        })
        .collect()
}

pub fn map_staking_delegations(delegations: Vec<DelegationBalance>, chain: Chain) -> Vec<DelegationBase> {
    delegations
        .into_iter()
        .map(|x| DelegationBase {
            asset_id: chain.as_asset_id(),
            state: DelegationState::Active,
            balance: BigNumberFormatter::value_from_amount(&x.amount.to_string(), 18)
                .ok()
                .and_then(|s| BigUint::from_str(&s).ok())
                .unwrap_or_default(),
            shares: BigUint::from(0u32),
            rewards: BigUint::from(0u32),
            completion_date: None,
            delegation_id: x.validator_address(),
            validator_id: x.validator_address(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::balance::ValidatorStats;
    use primitives::{Chain, DelegationState};

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
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Test Validator");
        assert_eq!(result[0].id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(result[0].chain, Chain::HyperCore);
        assert!(result[0].is_active);
        assert_eq!(result[0].commission, 5.0);
        assert_eq!(result[0].apr, 15.0); // max_apr * 100
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
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].apr, 10.0); // Uses provided APY
    }

    #[test]
    fn test_map_staking_delegations() {
        let delegations: Vec<DelegationBalance> = serde_json::from_str(include_str!("../../testdata/staking_delegations.json")).unwrap();

        let result = map_staking_delegations(delegations, Chain::HyperCore);

        assert_eq!(result.len(), 2);

        let delegation1 = &result[0];
        assert_eq!(delegation1.asset_id.chain, Chain::HyperCore);
        assert_eq!(delegation1.validator_id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(delegation1.delegation_id, "0x5aC99df645F3414876C816Caa18b2d234024b487");
        assert_eq!(delegation1.balance.to_string(), "2719364933730000000000");
        assert!(matches!(delegation1.state, DelegationState::Active));
        assert_eq!(delegation1.shares, num_bigint::BigUint::from(0u32));
        assert_eq!(delegation1.rewards, num_bigint::BigUint::from(0u32));
        assert!(delegation1.completion_date.is_none());

        let delegation2 = &result[1];
        assert_eq!(delegation2.validator_id, "0xaBCDefF4b3727B83A23697500EEf089020DF2cD2");
        assert_eq!(delegation2.balance.to_string(), "18145780860000000000");
    }
}
