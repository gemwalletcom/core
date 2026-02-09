use crate::earn_provider::EarnProviderType;
use crate::{AssetId, Chain, EarnPositionData, EarnPosition, EarnPositionState, EarnProvider};
use num_bigint::BigUint;

impl EarnPosition {
    pub fn mock() -> Self {
        EarnPosition {
            data: EarnPositionData::mock(),
            provider: EarnProvider::mock(),
            price: None,
        }
    }

    pub fn mock_with_id(position_id: String) -> Self {
        EarnPosition {
            data: EarnPositionData::mock_with_id(position_id),
            provider: EarnProvider::mock(),
            price: None,
        }
    }
}

impl EarnPositionData {
    pub fn mock() -> Self {
        EarnPositionData {
            asset_id: AssetId::from_chain(Chain::Sui),
            state: EarnPositionState::Active,
            balance: BigUint::from(1000000000u64),
            shares: BigUint::from(1000000000u64),
            rewards: BigUint::from(0u64),
            completion_date: None,
            position_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            provider_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }
    }

    pub fn mock_with_id(position_id: String) -> Self {
        EarnPositionData {
            asset_id: AssetId::from_chain(Chain::Sui),
            state: EarnPositionState::Active,
            balance: BigUint::from(1000000000u64),
            shares: BigUint::from(1000000000u64),
            rewards: BigUint::from(0u64),
            completion_date: None,
            position_id,
            provider_id: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
        }
    }
}

impl EarnProvider {
    pub fn mock() -> Self {
        EarnProvider {
            chain: Chain::Sui,
            id: "validator1".to_string(),
            name: "Test Validator".to_string(),
            is_active: true,
            commission: 0.05,
            apr: 0.08,
            provider_type: EarnProviderType::Stake,
        }
    }
}
