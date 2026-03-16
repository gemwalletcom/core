use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use primitives::{AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, YieldProvider};

use crate::error::YielderError;
use crate::provider::EarnProvider;

use super::{YO_PARTNER_ID_GEM, YoAsset, client::YoClient, mapper::{map_to_delegation, map_to_earn_provider}, supported_assets};

const GAS_LIMIT: u64 = 300_000;
const SLIPPAGE_BPS: u64 = 50;

pub struct YoEarnProvider {
    assets: &'static [YoAsset],
    gateways: HashMap<Chain, Arc<dyn YoClient>>,
}

impl YoEarnProvider {
    pub fn new(gateways: HashMap<Chain, Arc<dyn YoClient>>) -> Self {
        Self {
            assets: supported_assets(),
            gateways,
        }
    }

    fn asset(&self, asset_id: &AssetId) -> Result<YoAsset, YielderError> {
        self.assets
            .iter()
            .find(|a| a.asset_id() == *asset_id)
            .copied()
            .ok_or_else(|| YielderError::unsupported_asset(asset_id))
    }

    fn gateway_for_chain(&self, chain: Chain) -> Result<&Arc<dyn YoClient>, YielderError> {
        self.gateways.get(&chain).ok_or_else(|| YielderError::unsupported_chain(&chain))
    }

    async fn positions_for_chain(&self, chain: Chain, address: &str, assets: &[YoAsset]) -> Result<Vec<DelegationBase>, YielderError> {
        let gateway = self.gateway_for_chain(chain)?;
        let owner = Address::from_str(address)?;
        let provider_id = YieldProvider::Yo.to_string();
        let positions = gateway.positions_batch(assets, owner).await?;

        Ok(assets
            .iter()
            .zip(positions)
            .filter(|(_, data)| data.share_balance != U256::ZERO)
            .map(|(asset, data)| map_to_delegation(asset.asset_id(), &data, &provider_id))
            .collect())
    }
}

#[async_trait]
impl EarnProvider for YoEarnProvider {
    fn id(&self) -> YieldProvider {
        YieldProvider::Yo
    }

    fn get_providers(&self, asset_id: &AssetId) -> Vec<DelegationValidator> {
        self.assets.iter().filter(|a| a.asset_id() == *asset_id).map(|a| map_to_earn_provider(a.chain, YieldProvider::Yo)).collect()
    }

    fn get_asset_ids_for_chain(&self, chain: Chain) -> Vec<AssetId> {
        self.assets.iter().filter(|a| a.chain == chain).map(|a| a.asset_id()).collect()
    }

    async fn get_positions(&self, chain: Chain, address: &str, asset_ids: &[AssetId]) -> Result<Vec<DelegationBase>, YielderError> {
        let assets: Vec<_> = self.assets.iter().filter(|a| a.chain == chain && asset_ids.contains(&a.asset_id())).copied().collect();
        self.positions_for_chain(chain, address, &assets).await
    }

    async fn deposit(&self, asset_id: &AssetId, address: &str, value: &str) -> Result<ContractCallData, YielderError> {
        let asset = self.asset(asset_id)?;
        let gateway = self.gateway_for_chain(asset.chain)?;
        let wallet = Address::from_str(address)?;
        let amount = U256::from_str(value)?;

        let approval = gateway.check_token_allowance(asset.asset_token, wallet, amount).await?;
        let expected_shares = gateway.quote_shares(asset.yo_token, amount).await?;
        let min_shares_out = apply_slippage(expected_shares);
        let transaction = gateway.build_deposit_transaction(wallet, asset.yo_token, amount, min_shares_out, wallet, YO_PARTNER_ID_GEM);

        Ok(ContractCallData {
            contract_address: transaction.to,
            call_data: transaction.data,
            approval,
            gas_limit: Some(GAS_LIMIT.to_string()),
        })
    }

    async fn withdraw(&self, asset_id: &AssetId, address: &str, value: &str, shares: &str) -> Result<ContractCallData, YielderError> {
        let asset = self.asset(asset_id)?;
        let gateway = self.gateway_for_chain(asset.chain)?;
        let wallet = Address::from_str(address)?;
        let amount = U256::from_str(value)?;
        let total_shares = U256::from_str(shares)?;

        let computed_shares = gateway.quote_shares(asset.yo_token, amount).await?;
        let redeem_shares = if total_shares > computed_shares && total_shares - computed_shares <= U256::from(1) {
            total_shares
        } else {
            computed_shares.min(total_shares)
        };

        let approval = gateway.check_token_allowance(asset.yo_token, wallet, redeem_shares).await?;
        let min_assets_out = apply_slippage(amount);
        let transaction = gateway.build_redeem_transaction(wallet, asset.yo_token, redeem_shares, min_assets_out, wallet, YO_PARTNER_ID_GEM);

        Ok(ContractCallData {
            contract_address: transaction.to,
            call_data: transaction.data,
            approval,
            gas_limit: Some(GAS_LIMIT.to_string()),
        })
    }
}

fn apply_slippage(amount: U256) -> U256 {
    amount * U256::from(10_000 - SLIPPAGE_BPS) / U256::from(10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_slippage() {
        assert_eq!(apply_slippage(U256::from(10_000)), U256::from(9_950));
        assert_eq!(apply_slippage(U256::from(1_000_000)), U256::from(995_000));
        assert_eq!(apply_slippage(U256::ZERO), U256::ZERO);
    }
}
