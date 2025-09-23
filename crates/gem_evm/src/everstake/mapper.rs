use super::{WithdrawRequest, EVERSTAKE_POOL_ADDRESS};
use num_bigint::{BigInt, Sign};
use num_traits::Zero;
use primitives::{AssetId, Chain, DelegationBase, DelegationState};

fn delegation_id(validator_id: &str, state: DelegationState) -> String {
    format!("{}-{}", validator_id, state.as_ref())
}

pub fn map_withdraw_request_to_delegations(withdraw_request: &WithdrawRequest) -> Vec<DelegationBase> {
    let requested = BigInt::from_bytes_be(Sign::Plus, &withdraw_request.requested.to_be_bytes::<32>());
    let ready_for_claim = BigInt::from_bytes_be(Sign::Plus, &withdraw_request.readyForClaim.to_be_bytes::<32>());

    let mut delegations = Vec::new();
    let pending_amount = if requested > ready_for_claim {
        requested - &ready_for_claim
    } else {
        BigInt::zero()
    };

    let asset_id = AssetId::from_chain(Chain::Ethereum);
    let validator_id = EVERSTAKE_POOL_ADDRESS;

    if pending_amount > BigInt::zero() {
        delegations.push(DelegationBase {
            asset_id: asset_id.clone(),
            state: DelegationState::Deactivating,
            balance: pending_amount,
            shares: BigInt::zero(),
            rewards: BigInt::zero(),
            completion_date: None,
            delegation_id: delegation_id(validator_id, DelegationState::Deactivating),
            validator_id: validator_id.to_string(),
        });
    }

    if ready_for_claim > BigInt::zero() {
        delegations.push(DelegationBase {
            asset_id,
            state: DelegationState::AwaitingWithdrawal,
            balance: ready_for_claim,
            shares: BigInt::zero(),
            rewards: BigInt::zero(),
            completion_date: None,
            delegation_id: delegation_id(validator_id, DelegationState::AwaitingWithdrawal),
            validator_id: validator_id.to_string(),
        });
    }

    delegations
}

pub fn map_balance_to_delegation(balance: &BigInt, state: DelegationState) -> DelegationBase {
    DelegationBase {
        asset_id: AssetId::from_chain(Chain::Ethereum),
        state,
        balance: balance.clone(),
        shares: BigInt::zero(),
        rewards: BigInt::zero(),
        completion_date: None,
        delegation_id: delegation_id(EVERSTAKE_POOL_ADDRESS, state),
        validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
    use std::str::FromStr;

    #[test]
    fn test_map_withdraw_request_to_delegations() {
        let withdraw_request = WithdrawRequest {
            requested: U256::from_str_radix("1000000000000000000", 10).unwrap(),
            readyForClaim: U256::from_str_radix("500000000000000000", 10).unwrap(),
        };

        let delegations = map_withdraw_request_to_delegations(&withdraw_request);

        assert_eq!(delegations.len(), 2);

        let pending = delegations.iter().find(|d| matches!(d.state, DelegationState::Deactivating)).unwrap();
        assert_eq!(pending.balance, BigInt::from_str("500000000000000000").unwrap());
        assert_eq!(pending.delegation_id, delegation_id(EVERSTAKE_POOL_ADDRESS, DelegationState::Deactivating));

        let awaiting = delegations.iter().find(|d| matches!(d.state, DelegationState::AwaitingWithdrawal)).unwrap();
        assert_eq!(awaiting.balance, BigInt::from_str("500000000000000000").unwrap());
        assert_eq!(
            awaiting.delegation_id,
            delegation_id(EVERSTAKE_POOL_ADDRESS, DelegationState::AwaitingWithdrawal)
        );
    }
}
