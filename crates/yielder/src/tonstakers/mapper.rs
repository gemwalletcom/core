use num_bigint::BigUint;
use primitives::{AssetId, Chain, DelegationBase, DelegationState, YieldProvider};

pub fn map_to_delegation(balance: BigUint, shares: BigUint) -> DelegationBase {
    let provider = YieldProvider::Tonstakers.delegation_validator(Chain::Ton);
    let asset_id = AssetId::from_chain(Chain::Ton);

    DelegationBase {
        delegation_id: format!("{}-{}", provider.id, asset_id),
        validator_id: provider.id,
        asset_id,
        state: DelegationState::Active,
        balance,
        shares,
        rewards: BigUint::ZERO,
        completion_date: None,
    }
}

pub fn map_staked_balance(shares: &BigUint, total_balance: &BigUint, supply: &BigUint) -> BigUint {
    if supply == &BigUint::ZERO {
        return BigUint::ZERO;
    }
    (shares * total_balance) / supply
}

pub fn map_redeem_shares(amount: &BigUint, total_balance: &BigUint, supply: &BigUint, total_shares: &BigUint) -> BigUint {
    if total_balance == &BigUint::ZERO {
        return BigUint::ZERO;
    }

    let redeem_shares = (amount * supply) / total_balance;
    if total_shares > &redeem_shares && total_shares - &redeem_shares <= BigUint::from(1u8) {
        total_shares.clone()
    } else {
        redeem_shares.min(total_shares.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_staked_balance() {
        assert_eq!(
            map_staked_balance(&BigUint::from(1_000_000u64), &BigUint::from(1_050_000u64), &BigUint::from(1_000_000u64)),
            BigUint::from(1_050_000u64)
        );
        assert_eq!(
            map_staked_balance(&BigUint::from(1_000_000u64), &BigUint::from(0u8), &BigUint::from(1_000_000u64)),
            BigUint::ZERO
        );
        assert_eq!(
            map_staked_balance(&BigUint::from(1_000_000u64), &BigUint::from(1_050_000u64), &BigUint::ZERO),
            BigUint::ZERO
        );
    }

    #[test]
    fn test_map_redeem_shares() {
        assert_eq!(
            map_redeem_shares(
                &BigUint::from(1_050_000u64),
                &BigUint::from(1_050_000u64),
                &BigUint::from(1_000_000u64),
                &BigUint::from(1_000_000u64)
            ),
            BigUint::from(1_000_000u64)
        );
        assert_eq!(
            map_redeem_shares(
                &BigUint::from(525_000u64),
                &BigUint::from(1_050_000u64),
                &BigUint::from(1_000_000u64),
                &BigUint::from(1_000_000u64)
            ),
            BigUint::from(500_000u64)
        );
        assert_eq!(
            map_redeem_shares(
                &BigUint::from(1_050_000u64),
                &BigUint::from(1_050_001u64),
                &BigUint::from(1_000_000u64),
                &BigUint::from(1_000_000u64)
            ),
            BigUint::from(1_000_000u64)
        );
    }
}
