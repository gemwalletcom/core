use super::{WithdrawRequest, EVERSTAKE_POOL_ADDRESS};
use alloy_primitives::U256;
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{AssetId, Chain, DelegationBase, DelegationState};

pub fn map_withdraw_request_to_delegation(withdraw_request: &WithdrawRequest, balance: &BigInt) -> Option<DelegationBase> {
    if balance.is_zero() || withdraw_request.processed {
        return None;
    }

    let completion_date = if withdraw_request.requestTime > U256::ZERO {
        let timestamp = withdraw_request.requestTime.to::<u64>();
        DateTime::from_timestamp(timestamp as i64, 0).map(|dt| dt.with_timezone(&Utc))
    } else {
        None
    };

    Some(DelegationBase {
        asset_id: AssetId::from_chain(Chain::Ethereum),
        state: DelegationState::Undelegating,
        balance: balance.clone(),
        shares: BigInt::zero(),
        rewards: BigInt::zero(),
        completion_date,
        delegation_id: format!(
            "{}-{}-{}",
            EVERSTAKE_POOL_ADDRESS,
            DelegationState::Undelegating.as_ref(),
            withdraw_request.requestTime,
        ),
        validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
    })
}

pub fn map_balance_to_delegation(balance: &BigInt, rewards: &BigInt, state: DelegationState) -> DelegationBase {
    DelegationBase {
        asset_id: AssetId::from_chain(Chain::Ethereum),
        state,
        balance: balance.clone(),
        shares: BigInt::zero(),
        rewards: rewards.clone(),
        completion_date: None,
        delegation_id: format!("{}-{}", EVERSTAKE_POOL_ADDRESS, state.as_ref()),
        validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_map_withdraw_request_to_delegation() {
        let withdraw_request = WithdrawRequest {
            amount: U256::from_str_radix("1000000000000000000", 10).unwrap(),
            requestTime: U256::from(1234567890),
            processed: false,
        };

        let balance = BigInt::from_str("1000000000000000000").unwrap();
        let delegation = map_withdraw_request_to_delegation(&withdraw_request, &balance);

        assert!(delegation.is_some());
        let delegation = delegation.unwrap();
        assert_eq!(delegation.balance, BigInt::from_str("1000000000000000000").unwrap());
        assert!(matches!(delegation.state, DelegationState::Undelegating));
        assert!(delegation.completion_date.is_some());
        assert_eq!(
            delegation.delegation_id,
            format!("{}-undelegating-{}", EVERSTAKE_POOL_ADDRESS, withdraw_request.requestTime)
        );
    }
}
