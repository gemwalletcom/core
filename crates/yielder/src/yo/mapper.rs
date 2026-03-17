use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, DelegationBase, DelegationState, DelegationValidator, StakeProviderType, YieldProvider};

use super::client::PositionData;

pub fn map_to_delegation(asset_id: AssetId, data: &PositionData, provider_id: &str) -> DelegationBase {
    DelegationBase {
        delegation_id: format!("{}-{}", provider_id, asset_id),
        validator_id: provider_id.to_string(),
        asset_id,
        state: DelegationState::Active,
        balance: u256_to_biguint(&data.asset_balance),
        shares: u256_to_biguint(&data.share_balance),
        rewards: BigUint::ZERO,
        completion_date: None,
    }
}

pub fn map_to_earn_provider(chain: Chain, provider: YieldProvider) -> DelegationValidator {
    DelegationValidator {
        chain,
        id: provider.to_string(),
        name: provider.to_string(),
        is_active: true,
        commission: 0.0,
        apr: 0.0,
        provider_type: StakeProviderType::Earn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::assets::YO_USDC;
    use alloy_primitives::U256;

    #[test]
    fn test_map_to_delegation() {
        let data = PositionData {
            share_balance: U256::from(1_000_000),
            asset_balance: U256::from(1_050_000),
        };

        let result = map_to_delegation(YO_USDC.asset_id(), &data, "yo");

        assert_eq!(result.delegation_id, format!("yo-{}", YO_USDC.asset_id()));
        assert_eq!(result.balance, BigUint::from(1_050_000u64));
        assert_eq!(result.shares, BigUint::from(1_000_000u64));
    }

    #[test]
    fn test_map_to_earn_provider() {
        let result = map_to_earn_provider(Chain::Base, YieldProvider::Yo);

        assert_eq!(result.id, "yo");
        assert_eq!(result.chain, Chain::Base);
        assert_eq!(result.provider_type, StakeProviderType::Earn);
    }
}
