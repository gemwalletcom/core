use alloy_primitives::U256;
use gem_evm::jsonrpc::TransactionObject;
use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use primitives::swap::ApprovalData;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationState, DelegationValidator, StakeProviderType, YieldProvider};

use super::assets::YoAsset;
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

pub fn map_to_asset_balance(asset: &YoAsset, data: &PositionData) -> AssetBalance {
    let balance = if data.share_balance != U256::ZERO { 
        u256_to_biguint(&data.asset_balance) 
    } else {
         BigUint::ZERO 
    };
    AssetBalance::new_earn(asset.asset_id(), balance)
}

pub fn map_to_contract_call_data(transaction: TransactionObject, approval: Option<ApprovalData>, gas_limit: u64) -> ContractCallData {
    ContractCallData {
        contract_address: transaction.to,
        call_data: transaction.data,
        approval,
        gas_limit: Some(gas_limit.to_string()),
    }
}

pub fn map_to_earn_provider(chain: Chain, provider: YieldProvider) -> DelegationValidator {
    DelegationValidator {
        chain,
        id: provider.as_ref().to_string(),
        name: provider.name().to_string(),
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
    use primitives::AssetId;

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
        assert_eq!(result.name, "Yo");
        assert_eq!(result.chain, Chain::Base);
        assert_eq!(result.apr, 0.0);
        assert_eq!(result.provider_type, StakeProviderType::Earn);
    }

    #[test]
    fn test_map_to_asset_balance() {
        assert_eq!(map_to_asset_balance(&YO_USDC, &PositionData { share_balance: U256::from(1_000_000), asset_balance: U256::from(1_050_000) }).balance.earn, BigUint::from(1_050_000u64));
        assert_eq!(map_to_asset_balance(&YO_USDC, &PositionData { share_balance: U256::ZERO, asset_balance: U256::from(1_050_000) }).balance.earn, BigUint::ZERO);
    }

    #[test]
    fn test_map_to_contract_call_data() {
        let result = map_to_contract_call_data(TransactionObject { to: "0xcontract".to_string(), data: "0xcalldata".to_string() }, None, 300_000);
        assert_eq!(result.contract_address, "0xcontract");
        assert_eq!(result.call_data, "0xcalldata");
        assert_eq!(result.gas_limit, Some("300000".to_string()));
    }
}
