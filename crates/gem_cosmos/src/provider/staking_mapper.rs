use crate::models::staking::{Delegations, Rewards, UnbondingDelegations, StakingPoolResponse, Validator};
use crate::models::{InflationResponse, SupplyResponse, OsmosisEpochProvisionsResponse, OsmosisMintParamsResponse};

#[cfg(test)]
use crate::models::staking::{
    StakingPool, ValidatorCommission, ValidatorCommissionRates,
    ValidatorDescription, ValidatorsResponse,
};

#[cfg(test)]
use crate::models::{OsmosisDistributionProportions, OsmosisMintParams};

#[cfg(test)]
use crate::models::SupplyAmount;
use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::chain_cosmos::CosmosChain;
use primitives::{DelegationBase, DelegationState, DelegationValidator};
use std::collections::HashMap;
use std::str::FromStr;

const BOND_STATUS_BONDED: &str = "BOND_STATUS_BONDED";

pub fn calculate_network_apy_cosmos(inflation: InflationResponse, staking_pool: StakingPoolResponse) -> Option<f64> {
    let inflation_rate = inflation.inflation.parse::<f64>().unwrap_or(0.0);
    let bonded_tokens = staking_pool.pool.bonded_tokens.parse::<f64>().unwrap_or(1.0);
    let total_supply = bonded_tokens + staking_pool.pool.not_bonded_tokens.parse::<f64>().unwrap_or(0.0);

    let network_apy = inflation_rate * (total_supply / bonded_tokens);
    Some(network_apy * 100.0)
}

pub fn calculate_network_apy_osmosis(
    mint_params: OsmosisMintParamsResponse,
    epoch_provisions: OsmosisEpochProvisionsResponse,
    staking_pool: StakingPoolResponse,
    supply: SupplyResponse,
) -> Option<f64> {
    let epoch_provisions = epoch_provisions.epoch_provisions.parse::<f64>().ok()?;
    let staking_distribution = mint_params.params.distribution_proportions.staking.parse::<f64>().ok()?;
    let bonded_tokens = staking_pool.pool.bonded_tokens.parse::<f64>().ok()?;
    let _not_bonded_tokens = staking_pool.pool.not_bonded_tokens.parse::<f64>().ok()?;
    let _total_supply = supply.amount.amount.parse::<f64>().ok()?;

    let epochs_per_year = if mint_params.params.epoch_identifier == "day" { 365.0 } else { 52.0 };

    let annual_issuance = epoch_provisions * epochs_per_year;
    let effective_staking_distribution = if staking_distribution < 0.1 { 0.093 } else { staking_distribution };
    let annual_staking_rewards = annual_issuance * effective_staking_distribution;
    let staking_apy = (annual_staking_rewards / bonded_tokens) * 100.0;

    Some(staking_apy)
}

pub fn map_staking_validators(validators: Vec<Validator>, chain: CosmosChain, apy: Option<f64>) -> Vec<DelegationValidator> {
    validators
        .into_iter()
        .map(|validator| {
            let commission_rate = validator.commission.commission_rates.rate.parse::<f64>().unwrap_or(0.0);
            let is_active = !validator.jailed && validator.status == BOND_STATUS_BONDED;
            let validator_apr = if is_active {
                apy.map(|apr| apr - (apr * commission_rate)).unwrap_or(0.0)
            } else {
                0.0
            };

            DelegationValidator {
                chain: chain.as_chain(),
                id: validator.operator_address,
                name: validator.description.moniker,
                is_active,
                commision: commission_rate * 100.0, // Convert to percentage
                apr: validator_apr,
            }
        })
        .collect()
}

pub fn map_staking_delegations(
    active_delegations: Delegations,
    unbonding_delegations: UnbondingDelegations,
    rewards: Rewards,
    validators: Vec<Validator>,
    chain: CosmosChain,
    denom: &str,
) -> Vec<DelegationBase> {
    let asset_id = chain.as_chain().as_asset_id();
    let mut delegations = Vec::new();

    let validators_map: HashMap<String, &Validator> = validators.iter().map(|validator| (validator.operator_address.clone(), validator)).collect();

    let rewards_map: HashMap<String, BigInt> = rewards
        .rewards
        .iter()
        .map(|reward| {
            let total_reward = reward
                .reward
                .iter()
                .filter(|r| r.denom == denom)
                .filter_map(|r| {
                    let integer_part = r.amount.split('.').next().unwrap_or("0");
                    BigInt::from_str(integer_part).ok()
                })
                .fold(BigInt::from(0), |acc, amount| acc + amount);
            (reward.validator_address.clone(), total_reward)
        })
        .collect();

    let active_delegations = active_delegations.delegation_responses.into_iter().filter_map(|delegation| {
        let balance_value = BigNumberFormatter::value_from_amount(&delegation.balance.amount, 0).ok()?;
        if balance_value == "0" {
            return None;
        }

        let validator = validators_map.get(&delegation.delegation.validator_address);
        let state = if validator.map(|v| !v.jailed && v.status == BOND_STATUS_BONDED).unwrap_or(false) {
            DelegationState::Active
        } else {
            DelegationState::Inactive
        };

        let rewards = rewards_map
            .get(&delegation.delegation.validator_address)
            .map(|r| r.to_string())
            .unwrap_or_else(|| "0".to_string());

        Some(DelegationBase {
            asset_id: asset_id.clone(),
            state,
            balance: delegation.balance.amount,
            shares: "0".to_string(),
            rewards,
            completion_date: None,
            delegation_id: "".to_string(),
            validator_id: delegation.delegation.validator_address,
        })
    });
    delegations.extend(active_delegations);

    for unbonding in unbonding_delegations.unbonding_responses {
        for entry in unbonding.entries {
            let balance = BigInt::from_str(&entry.balance).unwrap_or_default().to_string();
            let rewards = rewards_map
                .get(&unbonding.validator_address)
                .map(|r| r.to_string())
                .unwrap_or_else(|| "0".to_string());

            delegations.push(DelegationBase {
                asset_id: asset_id.clone(),
                state: DelegationState::Pending,
                balance,
                shares: "0".to_string(),
                rewards,
                completion_date: entry.completion_time.parse::<chrono::DateTime<chrono::Utc>>().ok(),
                delegation_id: entry.creation_height,
                validator_id: unbonding.validator_address.clone(),
            });
        }
    }

    delegations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::staking::{Delegations, Rewards};
    use primitives::Chain;

    #[test]
    fn test_map_delegations() {
        let delegations: Delegations = serde_json::from_str(include_str!("../../testdata/staking_delegations.json")).unwrap();

        let mock_validator = Validator {
            operator_address: "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(),
            jailed: false,
            status: BOND_STATUS_BONDED.to_string(),
            description: ValidatorDescription {
                moniker: "Test Validator".to_string(),
            },
            commission: ValidatorCommission {
                commission_rates: ValidatorCommissionRates { rate: "0.05".to_string() },
            },
        };

        let unbonding = UnbondingDelegations { unbonding_responses: vec![] };
        let rewards = Rewards { rewards: vec![] };

        let result = map_staking_delegations(delegations, unbonding, rewards, vec![mock_validator], CosmosChain::Cosmos, "uatom");

        assert_eq!(result.len(), 1);
        let delegation = &result[0];
        assert_eq!(delegation.asset_id.to_string(), "cosmos");
        assert!(matches!(delegation.state, DelegationState::Active));
        assert_eq!(delegation.balance, "10250000");
        assert_eq!(delegation.validator_id, "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3");
        assert_eq!(delegation.rewards, "0");
        assert_eq!(delegation.shares, "0");
        assert!(delegation.completion_date.is_none());
        assert_eq!(delegation.delegation_id, "");
    }

    #[test]
    fn test_map_delegations_with_rewards() {
        let delegations: Delegations = serde_json::from_str(include_str!("../../testdata/staking_delegations.json")).unwrap();
        let rewards: Rewards = serde_json::from_str(include_str!("../../testdata/staking_rewards.json")).unwrap();

        let mock_validator = Validator {
            operator_address: "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3".to_string(),
            jailed: false,
            status: BOND_STATUS_BONDED.to_string(),
            description: ValidatorDescription {
                moniker: "Test Validator".to_string(),
            },
            commission: ValidatorCommission {
                commission_rates: ValidatorCommissionRates { rate: "0.05".to_string() },
            },
        };

        let unbonding = UnbondingDelegations { unbonding_responses: vec![] };

        let result = map_staking_delegations(delegations, unbonding, rewards, vec![mock_validator], CosmosChain::Cosmos, "uatom");

        assert_eq!(result.len(), 1);
        let delegation = &result[0];
        assert_eq!(delegation.asset_id.to_string(), "cosmos");
        assert!(matches!(delegation.state, DelegationState::Active));
        assert_eq!(delegation.balance, "10250000");
        assert_eq!(delegation.validator_id, "cosmosvaloper1tflk30mq5vgqjdly92kkhhq3raev2hnz6eete3");
        assert_eq!(delegation.rewards, "307413"); // Integer part of decimal amount
        assert_eq!(delegation.shares, "0");
        assert!(delegation.completion_date.is_none());
        assert_eq!(delegation.delegation_id, "");
    }

    #[test]
    fn test_map_validators() {
        let validators_response: ValidatorsResponse = serde_json::from_str(include_str!("../../testdata/staking_validators.json")).unwrap();

        let result = map_staking_validators(validators_response.validators, CosmosChain::Cosmos, Some(18.5));

        assert_eq!(result.len(), 2);

        let validator = &result[0];
        assert_eq!(validator.chain, Chain::Cosmos);
        assert_eq!(validator.id, "cosmosvaloper1q9p73lx07tjqc34vs8jrsu5pg3q4ha534uqv4w");
        assert_eq!(validator.name, "Unstake as we will shut down");
        assert!(validator.is_active);
        assert_eq!(validator.commision, 5.0); // Commission in percentage
        assert_eq!(validator.apr, 17.575);

        let validator2 = &result[1];
        assert_eq!(validator2.id, "cosmosvaloper1q6d3d089hg59x6gcx92uumx70s5y5wadklue8s");
        assert_eq!(validator2.name, "Ubik Capital");
    }

    #[test]
    fn test_calculate_network_apy_osmosis() {

        let mint_params = OsmosisMintParamsResponse {
            params: OsmosisMintParams {
                epoch_identifier: "day".to_string(),
                distribution_proportions: OsmosisDistributionProportions {
                    staking: "0.4".to_string(), // 40% of minted tokens go to staking
                },
            },
        };

        let epoch_provisions = OsmosisEpochProvisionsResponse {
            epoch_provisions: "100000000000".to_string(), // 100k OSMO per day
        };

        let staking_pool = StakingPoolResponse {
            pool: StakingPool {
                bonded_tokens: "400000000000000".to_string(),     // 400M OSMO bonded
                not_bonded_tokens: "100000000000000".to_string(), // 100M OSMO not bonded
            },
        };

        let supply = SupplyResponse {
            amount: SupplyAmount {
                denom: "uosmo".to_string(),
                amount: "1000000000000000".to_string(), // 1B OSMO total supply
            },
        };

        let result = calculate_network_apy_osmosis(mint_params, epoch_provisions, staking_pool, supply);

        assert!(result.is_some());
        let apy = result.unwrap();

        assert!((apy - 3.65).abs() < 0.001);
    }
}
