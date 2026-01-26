use std::{collections::HashMap, str::FromStr, sync::Arc};

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use gem_evm::jsonrpc::TransactionObject;
use gem_jsonrpc::RpcProvider;
use primitives::{AssetId, Chain, swap::ApprovalData};

use crate::models::{Yield, YieldDetailsRequest, YieldPosition, YieldProvider, YieldTransaction};
use crate::provider::YieldProviderClient;

use super::api::YoApiClient;
use super::{YO_PARTNER_ID_GEM, YoVault, client::YoProvider, error::YieldError, vaults};

pub const GAS_LIMIT: &str = "300000";

const SECONDS_PER_YEAR: f64 = 31_536_000.0;

fn lookback_blocks_for_chain(chain: Chain) -> u64 {
    match chain {
        Chain::Base => 7 * 24 * 60 * 60 / 2,
        Chain::Ethereum => 7 * 24 * 60 * 60 / 12,
        _ => 7 * 24 * 60 * 60 / 12,
    }
}

pub struct YoYieldProvider<E: std::error::Error + Send + Sync + 'static = YieldError> {
    vaults: Vec<YoVault>,
    gateways: HashMap<Chain, Arc<dyn YoProvider>>,
    api_client: YoApiClient<E>,
}

impl<E: std::error::Error + Send + Sync + 'static> YoYieldProvider<E> {
    pub fn new(gateways: HashMap<Chain, Arc<dyn YoProvider>>, rpc_provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        Self {
            vaults: vaults().to_vec(),
            gateways,
            api_client: YoApiClient::new(rpc_provider),
        }
    }

    fn get_vault(&self, asset_id: &AssetId) -> Result<YoVault, YieldError> {
        self.vaults_for_asset(asset_id).next().ok_or_else(|| format!("unsupported asset {}", asset_id).into())
    }

    fn vaults_for_asset(&self, asset_id: &AssetId) -> impl Iterator<Item = YoVault> + '_ {
        let asset_id = asset_id.clone();
        self.vaults.iter().copied().filter(move |vault| vault.asset_id() == asset_id)
    }

    fn gateway_for_chain(&self, chain: Chain) -> Result<&Arc<dyn YoProvider>, YieldError> {
        self.gateways.get(&chain).ok_or_else(|| format!("no gateway configured for chain {:?}", chain).into())
    }
}

#[async_trait]
impl<E: std::error::Error + Send + Sync + 'static> YieldProviderClient for YoYieldProvider<E> {
    fn provider(&self) -> YieldProvider {
        YieldProvider::Yo
    }

    fn yields(&self, asset_id: &AssetId) -> Vec<Yield> {
        self.vaults_for_asset(asset_id)
            .map(|vault| Yield::new(vault.name, vault.asset_id(), self.provider(), None))
            .collect()
    }

    async fn yields_with_apy(&self, asset_id: &AssetId) -> Result<Vec<Yield>, YieldError> {
        let mut results = Vec::new();

        for vault in self.vaults_for_asset(asset_id) {
            let gateway = self.gateway_for_chain(vault.chain)?;
            let lookback_blocks = lookback_blocks_for_chain(vault.chain);
            let data = gateway.get_position(vault, Address::ZERO, lookback_blocks).await?;
            let elapsed = data.latest_timestamp.saturating_sub(data.lookback_timestamp);
            let apy = annualize_growth(data.latest_price, data.lookback_price, elapsed);
            results.push(Yield::new(vault.name, vault.asset_id(), self.provider(), apy));
        }

        Ok(results)
    }

    async fn deposit(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let vault = self.get_vault(asset_id)?;
        let gateway = self.gateway_for_chain(vault.chain)?;
        let wallet = Address::from_str(wallet_address).map_err(|e| format!("invalid address {wallet_address}: {e}"))?;
        let amount = U256::from_str_radix(value, 10).map_err(|e| format!("invalid value {value}: {e}"))?;

        let approval = gateway.check_token_allowance(vault.asset_token, wallet, amount).await?;
        let tx = gateway.build_deposit_transaction(wallet, vault.yo_token, amount, U256::ZERO, wallet, YO_PARTNER_ID_GEM);
        Ok(convert_transaction(vault, tx, approval))
    }

    async fn withdraw(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let vault = self.get_vault(asset_id)?;
        let gateway = self.gateway_for_chain(vault.chain)?;
        let wallet = Address::from_str(wallet_address).map_err(|e| format!("invalid address {wallet_address}: {e}"))?;
        let assets = U256::from_str_radix(value, 10).map_err(|e| format!("invalid value {value}: {e}"))?;

        let shares = gateway.convert_to_shares(vault.yo_token, assets).await?;
        let approval = gateway.check_token_allowance(vault.yo_token, wallet, shares).await?;
        let tx = gateway.build_redeem_transaction(wallet, vault.yo_token, shares, U256::ZERO, wallet, YO_PARTNER_ID_GEM);
        Ok(convert_transaction(vault, tx, approval))
    }

    async fn positions(&self, request: &YieldDetailsRequest) -> Result<YieldPosition, YieldError> {
        let vault = self.get_vault(&request.asset_id)?;
        let gateway = self.gateway_for_chain(vault.chain)?;
        let owner = Address::from_str(&request.wallet_address).map_err(|e| format!("invalid address {}: {e}", request.wallet_address))?;
        let data = gateway.get_position(vault, owner, lookback_blocks_for_chain(vault.chain)).await?;

        let one_share = U256::from(10u64).pow(U256::from(vault.asset_decimals));
        let asset_value = data.share_balance.saturating_mul(data.latest_price) / one_share;
        let elapsed = data.latest_timestamp.saturating_sub(data.lookback_timestamp);
        let apy = annualize_growth(data.latest_price, data.lookback_price, elapsed);

        let rewards = self
            .api_client
            .fetch_rewards(vault.chain, &vault.yo_token.to_string(), &request.wallet_address)
            .await
            .ok()
            .map(|p| p.total_rewards_raw().to_string());

        Ok(YieldPosition {
            name: vault.name.to_string(),
            asset_id: request.asset_id.clone(),
            provider: self.provider(),
            vault_token_address: vault.yo_token.to_string(),
            asset_token_address: vault.asset_token.to_string(),
            vault_balance_value: Some(data.share_balance.to_string()),
            asset_balance_value: Some(asset_value.to_string()),
            apy,
            rewards,
        })
    }
}

fn convert_transaction(vault: YoVault, tx: TransactionObject, approval: Option<ApprovalData>) -> YieldTransaction {
    YieldTransaction {
        chain: vault.chain,
        from: tx.from.unwrap_or_default(),
        to: tx.to,
        data: tx.data,
        value: tx.value,
        approval,
    }
}

fn annualize_growth(latest_assets: U256, previous_assets: U256, elapsed_seconds: u64) -> Option<f64> {
    if elapsed_seconds == 0 || previous_assets.is_zero() {
        return None;
    }

    let latest = u256_to_f64(latest_assets)?;
    let previous = u256_to_f64(previous_assets)?;
    if latest <= 0.0 || previous <= 0.0 {
        return None;
    }

    let growth = latest / previous;
    if !growth.is_finite() || growth <= 0.0 {
        return None;
    }

    Some(growth.powf(SECONDS_PER_YEAR / elapsed_seconds as f64) - 1.0)
}

fn u256_to_f64(value: U256) -> Option<f64> {
    value.to_string().parse::<f64>().ok()
}
