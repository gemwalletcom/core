use crate::models::balance::{HypercoreDelegationBalance, HypercoreValidator};
use number_formatter::BigNumberFormatter;
use primitives::{Chain, DelegationBase, DelegationState, DelegationValidator};

pub fn map_validators_to_delegation_validators(validators: Vec<HypercoreValidator>, chain: Chain, apy: Option<f64>) -> Vec<DelegationValidator> {
    let calculated_apy = apy.unwrap_or_else(|| HypercoreValidator::max_apr(validators.clone()));
    validators
        .into_iter()
        .map(|x| DelegationValidator {
            chain,
            id: x.validator_address(),
            name: x.name,
            is_active: x.is_active,
            commision: x.commission.parse::<f64>().unwrap_or(0.0),
            apr: calculated_apy,
        })
        .collect()
}

pub fn map_delegations_to_delegation_bases(delegations: Vec<HypercoreDelegationBalance>, chain: Chain) -> Vec<DelegationBase> {
    delegations
        .into_iter()
        .map(|x| DelegationBase {
            asset_id: chain.as_asset_id(),
            state: DelegationState::Active,
            balance: BigNumberFormatter::value_from_amount(&x.amount, 18).unwrap_or("0".to_string()),
            shares: "0".to_string(),
            rewards: "0".to_string(),
            completion_date: None,
            delegation_id: x.validator_address(),
            validator_id: x.validator_address(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_map_validators_to_delegation_validators() {
        let validators = vec![HypercoreValidator {
            validator: "0x123".to_string(),
            name: "Test Validator".to_string(),
            commission: "5.0".to_string(),
            is_active: true,
            stats: vec![],
        }];

        let result = map_validators_to_delegation_validators(validators, Chain::SmartChain, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Test Validator");
        assert!(result[0].is_active);
    }
}
