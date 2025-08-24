use crate::models::staking::{Delegations, Rewards, UnbondingDelegations};
use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::AssetBalance;
use std::str::FromStr;

pub fn map_balance_staking(
    delegations: Delegations,
    unbonding: UnbondingDelegations,
    rewards: Rewards,
    chain: primitives::Chain,
    denom: &str,
) -> AssetBalance {
    let staked = delegations
        .delegation_responses
        .iter()
        .filter(|d| d.balance.denom == denom)
        .filter_map(|d| BigNumberFormatter::value_from_amount(&d.balance.amount, 0).ok())
        .filter_map(|v| BigInt::from_str(&v).ok())
        .fold(BigInt::from(0), |acc, amount| acc + amount);

    let pending = unbonding
        .unbonding_responses
        .iter()
        .flat_map(|u| &u.entries)
        .filter_map(|entry| BigNumberFormatter::value_from_amount(&entry.balance, 0).ok())
        .filter_map(|v| BigInt::from_str(&v).ok())
        .fold(BigInt::from(0), |acc, amount| acc + amount);

    let rewards = rewards
        .rewards
        .iter()
        .flat_map(|r| &r.reward)
        .filter(|r| r.denom == denom)
        .filter_map(|r| {
            let integer_part = r.amount.split('.').next().unwrap_or("0");
            BigInt::from_str(integer_part).ok()
        })
        .fold(BigInt::from(0), |acc, amount| acc + amount);

    AssetBalance::new_staking(chain.as_asset_id(), staked.to_string(), pending.to_string(), rewards.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::staking::{Delegations, Rewards, UnbondingDelegations};
    use primitives::Chain;

    #[test]
    fn test_map_balance_staking() {
        let delegations: Delegations = serde_json::from_str(include_str!("../../testdata/staking_delegations.json")).unwrap();
        let unbonding: UnbondingDelegations = serde_json::from_str(r#"{"unbonding_responses": []}"#).unwrap();
        let rewards: Rewards = serde_json::from_str(include_str!("../../testdata/staking_rewards.json")).unwrap();

        let result = map_balance_staking(delegations, unbonding, rewards, Chain::Cosmos, "uatom");

        assert_eq!(result.asset_id.to_string(), "cosmos");
        assert_eq!(result.balance.staked, "10250000");
        assert_eq!(result.balance.pending, "0");
        assert_eq!(result.balance.rewards, "307413");
    }
}
