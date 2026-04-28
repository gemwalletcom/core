use std::slice::from_ref;
use std::sync::Arc;

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use gem_evm::slippage::apply_slippage_in_bp;
use gem_evm::u256::biguint_to_u256;
use gem_jsonrpc::alien::RpcProvider;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType, YieldProvider};

use crate::client_factory::create_eth_client;
use crate::error::YielderError;
use crate::provider::EarnProvider;

use super::client::YoGatewayClient;
use super::mapper::{map_to_asset_balance, map_to_contract_call_data, map_to_delegation};
use super::{YO_GATEWAY, YO_PARTNER_ID_GEM, YoAsset, supported_assets};

const GAS_LIMIT: u64 = 300_000;
const SLIPPAGE_BPS: u32 = 50;

pub struct YoEarnProvider {
    assets: &'static [YoAsset],
    rpc_provider: Arc<dyn RpcProvider>,
}

impl YoEarnProvider {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            assets: supported_assets(),
            rpc_provider,
        }
    }

    fn get_assets(&self, chain: Chain, token_ids: &[String]) -> Vec<YoAsset> {
        self.assets
            .iter()
            .filter(|a| a.chain == chain && token_ids.contains(&a.asset_token.to_string()))
            .copied()
            .collect()
    }

    fn get_asset(&self, asset_id: &AssetId) -> Result<YoAsset, YielderError> {
        self.assets.iter().find(|a| a.asset_id() == *asset_id).copied().ok_or(YielderError::NotSupportedAsset)
    }

    fn get_client(&self, chain: Chain) -> Result<YoGatewayClient, YielderError> {
        let client = create_eth_client(self.rpc_provider.clone(), chain)?;
        Ok(YoGatewayClient::new(client, YO_GATEWAY))
    }

    async fn get_positions(&self, chain: Chain, address: &str, assets: &[YoAsset]) -> Result<Vec<super::client::PositionData>, YielderError> {
        let client = self.get_client(chain)?;
        let owner: Address = address.parse()?;
        client.get_positions(assets, owner).await
    }
}

#[async_trait]
impl EarnProvider for YoEarnProvider {
    fn get_provider(&self, asset_id: &AssetId) -> Option<DelegationValidator> {
        self.get_asset(asset_id).ok().map(|a| YieldProvider::Yo.delegation_validator(a.chain))
    }

    async fn get_position(&self, address: &str, asset_id: &AssetId) -> Result<Option<DelegationBase>, YielderError> {
        let asset = self.get_asset(asset_id)?;
        let positions = self.get_positions(asset.chain, address, from_ref(&asset)).await?;
        let delegation = positions
            .into_iter()
            .find(|d| d.share_balance != U256::ZERO)
            .map(|data| map_to_delegation(asset.asset_id(), &data, YieldProvider::Yo.as_ref()));
        Ok(delegation)
    }

    async fn get_balance(&self, chain: Chain, address: &str, token_ids: &[String]) -> Result<Vec<AssetBalance>, YielderError> {
        let assets = self.get_assets(chain, token_ids);
        if assets.is_empty() {
            return Ok(vec![]);
        }
        let positions = self.get_positions(chain, address, &assets).await?;
        let balances = assets.iter().zip(positions).map(|(asset, data)| map_to_asset_balance(asset, &data)).collect();
        Ok(balances)
    }

    async fn get_data(&self, asset_id: &AssetId, address: &str, value: &str, earn_type: &EarnType) -> Result<ContractCallData, YielderError> {
        let asset = self.get_asset(asset_id)?;
        let client = self.get_client(asset.chain)?;
        let wallet: Address = address.parse()?;
        let amount: U256 = value.parse()?;

        let (approval, transaction) = match earn_type {
            EarnType::Deposit(_) => {
                let approval = client.check_token_allowance(asset.asset_token, wallet, amount).await?;
                let expected_shares = client.get_quote_shares(asset.yo_token, amount).await?;
                let min_shares_out = apply_slippage_in_bp(&expected_shares, SLIPPAGE_BPS);
                let transaction = client.build_deposit_transaction(wallet, asset.yo_token, amount, min_shares_out, wallet, YO_PARTNER_ID_GEM);
                (approval, transaction)
            }
            EarnType::Withdraw(delegation) => {
                let total_shares = biguint_to_u256(&delegation.base.shares).ok_or_else(|| YielderError::invalid_input("invalid shares"))?;
                let computed_shares = client.get_quote_shares(asset.yo_token, amount).await?;
                let redeem_shares = if total_shares > computed_shares && total_shares - computed_shares <= U256::from(1) {
                    total_shares
                } else {
                    computed_shares.min(total_shares)
                };
                let approval = client.check_token_allowance(asset.yo_token, wallet, redeem_shares).await?;
                let min_assets_out = apply_slippage_in_bp(&amount, SLIPPAGE_BPS);
                let transaction = client.build_redeem_transaction(wallet, asset.yo_token, redeem_shares, min_assets_out, wallet, YO_PARTNER_ID_GEM);
                (approval, transaction)
            }
        };

        Ok(map_to_contract_call_data(transaction, approval, GAS_LIMIT))
    }
}
