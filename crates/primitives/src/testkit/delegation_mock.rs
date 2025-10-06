use crate::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator};
use num_bigint::BigUint;

impl Delegation {
    pub fn mock() -> Self {
        Delegation {
            base: DelegationBase::mock(),
            validator: DelegationValidator::mock(),
            price: None,
        }
    }

    pub fn mock_with_id(delegation_id: String) -> Self {
        Delegation {
            base: DelegationBase::mock_with_id(delegation_id),
            validator: DelegationValidator::mock(),
            price: None,
        }
    }
}

impl DelegationBase {
    pub fn mock() -> Self {
        DelegationBase {
            asset_id: AssetId::from_chain(Chain::Sui),
            state: DelegationState::Active,
            balance: BigUint::from(1000000000u64),
            shares: BigUint::from(1000000000u64),
            rewards: BigUint::from(0u64),
            completion_date: None,
            delegation_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            validator_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }
    }

    pub fn mock_with_id(delegation_id: String) -> Self {
        DelegationBase {
            asset_id: AssetId::from_chain(Chain::Sui),
            state: DelegationState::Active,
            balance: BigUint::from(1000000000u64),
            shares: BigUint::from(1000000000u64),
            rewards: BigUint::from(0u64),
            completion_date: None,
            delegation_id,
            validator_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }
    }
}

impl DelegationValidator {
    pub fn mock() -> Self {
        DelegationValidator {
            chain: Chain::Sui,
            id: "validator1".to_string(),
            name: "Test Validator".to_string(),
            is_active: true,
            commission: 0.05,
            apr: 0.08,
        }
    }
}
