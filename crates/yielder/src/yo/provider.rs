use std::collections::HashMap;
use std::sync::Arc;

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain, ContractCallData, DelegationBase, DelegationValidator, EarnType, YieldProvider};

use gem_evm::slippage::apply_slippage_in_bp;
use gem_evm::u256::biguint_to_u256;

use crate::client_factory::create_eth_client;
use crate::error::YielderError;
use crate::provider::EarnProvider;

use super::{YO_GATEWAY, YO_PARTNER_ID_GEM, YoAsset, client::{YoClient, YoGatewayClient}, mapper::{map_to_delegation, map_to_earn_provider}, supported_assets};

const GAS_LIMIT: u64 = 300_000;
const SLIPPAGE_BPS: u32 = 50;

pub struct YoEarnProvider {
    assets: &'static [YoAsset],
    clients: HashMap<Chain, Arc<dyn YoClient>>,
}

impl YoEarnProvider {
    pub fn new<E: gem_jsonrpc::RpcClientError + Clone + 'static>(rpc_provider: Arc<dyn gem_jsonrpc::RpcProvider<Error = E>>) -> Self {
        let assets = supported_assets();
        let clients = assets
            .iter()
            .filter_map(|asset| {
                let client = create_eth_client(rpc_provider.clone(), asset.chain).ok()?;
                Some((asset.chain, Arc::new(YoGatewayClient::new(client, YO_GATEWAY)) as Arc<dyn YoClient>))
            })
            .collect();
        Self { assets, clients }
    }

    fn get_asset(&self, asset_id: &AssetId) -> Result<YoAsset, YielderError> {
        self.assets
            .iter()
            .find(|a| a.asset_id() == *asset_id)
            .copied()
            .ok_or_else(|| YielderError::unsupported_asset(asset_id))
    }

    fn get_client(&self, chain: Chain) -> Result<&Arc<dyn YoClient>, YielderError> {
        self.clients.get(&chain).ok_or_else(|| YielderError::unsupported_chain(&chain))
    }

    async fn get_position_for_asset(&self, address: &str, asset: &YoAsset) -> Result<Option<DelegationBase>, YielderError> {
        let client = self.get_client(asset.chain)?;
        let owner: Address = address.parse()?;
        let positions = client.get_positions(&[*asset], owner).await?;

        Ok(positions.into_iter().find(|d| d.share_balance != U256::ZERO).map(|data| map_to_delegation(asset.asset_id(), &data, YieldProvider::Yo.as_ref())))
    }
}

#[async_trait]
impl EarnProvider for YoEarnProvider {
    fn get_provider(&self, asset_id: &AssetId) -> Option<DelegationValidator> {
        self.assets.iter().find(|a| a.asset_id() == *asset_id).map(|a| map_to_earn_provider(a.chain, YieldProvider::Yo))
    }

    async fn get_position(&self, address: &str, asset_id: &AssetId) -> Result<Option<DelegationBase>, YielderError> {
        let asset = self.get_asset(asset_id)?;
        self.get_position_for_asset(address, &asset).await
    }

    async fn get_balance(&self, chain: Chain, address: &str) -> Result<Vec<AssetBalance>, YielderError> {
        let chain_assets: Vec<_> = self.assets.iter().filter(|a| a.chain == chain).collect();
        let futures: Vec<_> = chain_assets.iter().map(|a| self.get_position_for_asset(address, a)).collect();
        Ok(chain_assets
            .iter()
            .zip(futures::future::join_all(futures).await)
            .map(|(asset, position)| {
                let balance = position.ok().flatten().map(|p| p.balance).unwrap_or(BigUint::ZERO);
                AssetBalance::new_earn(asset.asset_id(), balance)
            })
            .collect())
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
                let total_shares = biguint_to_u256(&delegation.base.shares).ok_or_else(|| YielderError::NetworkError("Invalid shares".to_string()))?;
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

        Ok(ContractCallData {
            contract_address: transaction.to,
            call_data: transaction.data,
            approval,
            gas_limit: Some(GAS_LIMIT.to_string()),
        })
    }
}
