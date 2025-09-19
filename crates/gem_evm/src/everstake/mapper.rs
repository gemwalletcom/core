use crate::everstake::WithdrawRequest;
use alloy_primitives::U256;
use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use num_traits::Zero;
use primitives::{AssetId, Chain, DelegationBase, DelegationState};
use std::str::FromStr;

pub fn map_deposited_balance_to_delegation(validator_id: &str, balance: &str, state: DelegationState) -> Option<DelegationBase> {
    if balance == "0" {
        return None;
    }

    Some(DelegationBase {
        asset_id: AssetId {
            chain: Chain::Ethereum,
            token_id: None,
        },
        state,
        balance: BigInt::from_str(balance).unwrap_or_default(),
        shares: BigInt::zero(),
        rewards: BigInt::default(),
        completion_date: None,
        delegation_id: format!("{}-{}", validator_id, state.as_ref()),
        validator_id: validator_id.to_string(),
    })
}

pub fn map_withdraw_request_to_delegation(validator_id: &str, withdraw_request: &WithdrawRequest) -> Option<DelegationBase> {
    if withdraw_request.amount == U256::ZERO || withdraw_request.processed {
        return None;
    }

    let completion_date = if withdraw_request.requestTime > U256::ZERO {
        let timestamp = withdraw_request.requestTime.to::<u64>();
        DateTime::from_timestamp(timestamp as i64, 0).map(|dt| dt.with_timezone(&Utc))
    } else {
        None
    };

    Some(DelegationBase {
        asset_id: AssetId {
            chain: Chain::Ethereum,
            token_id: None,
        },
        state: DelegationState::Undelegating,
        balance: BigInt::from_str(&withdraw_request.amount.to_string()).unwrap_or_default(),
        shares: BigInt::from_str(&withdraw_request.amount.to_string()).unwrap_or_default(),
        rewards: BigInt::default(),
        completion_date,
        delegation_id: format!("{}-{}", validator_id, DelegationState::Undelegating.as_ref()),
        validator_id: validator_id.to_string(),
    })
}

pub fn map_balance_string_to_delegation(validator_id: &str, balance: &str, state: DelegationState) -> Option<DelegationBase> {
    if balance == "0" || balance.is_empty() {
        return None;
    }

    Some(DelegationBase {
        asset_id: AssetId {
            chain: Chain::Ethereum,
            token_id: None,
        },
        state,
        balance: BigInt::from_str(balance).unwrap_or_default(),
        shares: BigInt::from_str(balance).unwrap_or_default(),
        rewards: BigInt::default(),
        completion_date: None,
        delegation_id: format!("{}-{}", validator_id, state.as_ref()),
        validator_id: validator_id.to_string(),
    })
}

pub fn combine_active_balances(deposited: &str, autocompound: &str) -> Option<String> {
    let deposited_val = deposited.parse::<u128>().unwrap_or(0);
    let autocompound_val = autocompound.parse::<u128>().unwrap_or(0);

    let total = deposited_val + autocompound_val;

    if total > 0 {
        Some(total.to_string())
    } else {
        None
    }
}

pub fn is_valid_balance(balance: &str) -> bool {
    !balance.is_empty() && balance != "0" && balance.parse::<u128>().unwrap_or(0) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_deposited_balance_to_delegation() {
        let delegation = map_deposited_balance_to_delegation("0x123", "1000000000000000000", DelegationState::Active);

        assert!(delegation.is_some());
        let delegation = delegation.unwrap();
        assert_eq!(delegation.validator_id, "0x123");
        assert_eq!(delegation.balance, BigInt::from_str("1000000000000000000").unwrap());
        assert!(matches!(delegation.state, DelegationState::Active));
        assert_eq!(delegation.asset_id.chain, Chain::Ethereum);
        assert_eq!(delegation.delegation_id, "0x123-active");

        // Test zero balance
        let empty_delegation = map_deposited_balance_to_delegation("0x123", "0", DelegationState::Active);
        assert!(empty_delegation.is_none());
    }

    #[test]
    fn test_map_withdraw_request_to_delegation() {
        let withdraw_request = WithdrawRequest {
            amount: U256::from_str_radix("1000000000000000000", 10).unwrap(),
            requestTime: U256::from(1234567890),
            processed: false,
        };

        let delegation = map_withdraw_request_to_delegation("0x456", &withdraw_request);

        assert!(delegation.is_some());
        let delegation = delegation.unwrap();
        assert_eq!(delegation.validator_id, "0x456");
        assert_eq!(delegation.balance, BigInt::from_str("1000000000000000000").unwrap());
        assert!(matches!(delegation.state, DelegationState::Undelegating));
        assert!(delegation.completion_date.is_some());
        assert_eq!(delegation.delegation_id, "0x456-undelegating");
    }

    #[test]
    fn test_combine_active_balances() {
        let combined = combine_active_balances("1000000000000000000", "500000000000000000");
        assert_eq!(combined, Some("1500000000000000000".to_string()));

        let zero_combined = combine_active_balances("0", "0");
        assert_eq!(zero_combined, None);
    }

    #[test]
    fn test_is_valid_balance() {
        assert!(is_valid_balance("1000000000000000000"));
        assert!(!is_valid_balance("0"));
        assert!(!is_valid_balance(""));
    }
}
