use std::{collections::HashMap, str::FromStr, sync::Arc};

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use gem_evm::jsonrpc::TransactionObject;
use primitives::{AssetId, Chain};

use crate::models::{Yield, YieldDetailsRequest, YieldPosition, YieldProvider, YieldTransaction};
use crate::provider::YieldProviderClient;

use super::{YO_PARTNER_ID_GEM, YoVault, client::YoProvider, error::YieldError, vaults};

const SECONDS_PER_YEAR: f64 = 31_536_000.0;

fn lookback_blocks_for_chain(chain: Chain) -> u64 {
    match chain {
        // Base chain has ~2 second block time, 7 days lookback
        Chain::Base => 7 * 24 * 60 * 60 / 2,
        // Ethereum has ~12 second block time, 7 days lookback
        Chain::Ethereum => 7 * 24 * 60 * 60 / 12,
        _ => 7 * 24 * 60 * 60 / 12, // Default to Ethereum-like
    }
}

#[derive(Clone)]
pub struct YoYieldProvider {
    vaults: Vec<YoVault>,
    gateways: HashMap<Chain, Arc<dyn YoProvider>>,
}

impl YoYieldProvider {
    pub fn new(gateways: HashMap<Chain, Arc<dyn YoProvider>>) -> Self {
        Self {
            vaults: vaults().to_vec(),
            gateways,
        }
    }

    fn find_vault(&self, asset_id: &AssetId) -> Result<YoVault, YieldError> {
        self.vaults
            .iter()
            .copied()
            .find(|vault| vault.asset_id() == *asset_id)
            .ok_or_else(|| YieldError::new(format!("unsupported asset {}", asset_id)))
    }

    fn gateway_for_chain(&self, chain: Chain) -> Result<&Arc<dyn YoProvider>, YieldError> {
        self.gateways
            .get(&chain)
            .ok_or_else(|| YieldError::new(format!("no gateway configured for chain {:?}", chain)))
    }
}

#[async_trait]
impl YieldProviderClient for YoYieldProvider {
    fn provider(&self) -> YieldProvider {
        YieldProvider::Yo
    }

    fn yields(&self, asset_id: &AssetId) -> Vec<Yield> {
        self.vaults
            .iter()
            .filter_map(|vault| {
                let vault_asset = vault.asset_id();
                if &vault_asset == asset_id {
                    Some(Yield::new(vault.name, vault_asset, self.provider(), None))
                } else {
                    None
                }
            })
            .collect()
    }

    async fn yields_with_apy(&self, asset_id: &AssetId) -> Result<Vec<Yield>, YieldError> {
        let mut results = Vec::new();

        for vault in self.vaults.iter().copied().filter(|vault| vault.asset_id() == *asset_id) {
            let gateway = self.gateway_for_chain(vault.chain)?;
            let lookback_blocks = lookback_blocks_for_chain(vault.chain);
            let data = gateway.fetch_position_data(vault, Address::ZERO, lookback_blocks).await?;
            let elapsed = data.latest_timestamp.saturating_sub(data.lookback_timestamp);
            let apy = annualize_growth(data.latest_price, data.lookback_price, elapsed);
            results.push(Yield::new(vault.name, vault.asset_id(), self.provider(), apy));
        }

        Ok(results)
    }

    async fn deposit(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let vault = self.find_vault(asset_id)?;
        let gateway = self.gateway_for_chain(vault.chain)?;
        let wallet = parse_address(wallet_address)?;
        let receiver = wallet;
        let amount = parse_value(value)?;
        let min_shares = U256::from(0);
        let partner_id = YO_PARTNER_ID_GEM;

        let tx = gateway.build_deposit_transaction(wallet, vault.yo_token, amount, min_shares, receiver, partner_id);
        Ok(convert_transaction(vault, tx))
    }

    async fn withdraw(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let vault = self.find_vault(asset_id)?;
        let gateway = self.gateway_for_chain(vault.chain)?;
        let wallet = parse_address(wallet_address)?;
        let receiver = wallet;
        let shares = parse_value(value)?;
        let min_assets = U256::from(0);
        let partner_id = YO_PARTNER_ID_GEM;

        let tx = gateway.build_redeem_transaction(wallet, vault.yo_token, shares, min_assets, receiver, partner_id);
        Ok(convert_transaction(vault, tx))
    }

    async fn positions(&self, request: &YieldDetailsRequest) -> Result<YieldPosition, YieldError> {
        let vault = self.find_vault(&request.asset_id)?;
        let gateway = self.gateway_for_chain(vault.chain)?;
        let lookback_blocks = lookback_blocks_for_chain(vault.chain);
        let owner = parse_address(&request.wallet_address)?;
        let mut details = YieldPosition::new(vault.name, request.asset_id.clone(), self.provider(), vault.yo_token, vault.asset_token);

        let data = gateway.fetch_position_data(vault, owner, lookback_blocks).await?;

        details.vault_balance_value = Some(data.share_balance.to_string());

        // Calculate asset value from shares: share_balance * latest_price / one_share
        let one_share = U256::from(10u64).pow(U256::from(vault.asset_decimals));
        let asset_value = data.share_balance.saturating_mul(data.latest_price) / one_share;
        details.asset_balance_value = Some(asset_value.to_string());

        let elapsed = data.latest_timestamp.saturating_sub(data.lookback_timestamp);
        details.apy = annualize_growth(data.latest_price, data.lookback_price, elapsed);

        Ok(details)
    }
}

fn parse_address(value: &str) -> Result<Address, YieldError> {
    Address::from_str(value).map_err(|err| YieldError::new(format!("invalid address {value}: {err}")))
}

fn parse_value(value: &str) -> Result<U256, YieldError> {
    U256::from_str_radix(value, 10).map_err(|err| YieldError::new(format!("invalid value {value}: {err}")))
}

fn convert_transaction(vault: YoVault, tx: TransactionObject) -> YieldTransaction {
    YieldTransaction {
        chain: vault.chain,
        from: tx.from.unwrap_or_default(),
        to: tx.to,
        data: tx.data,
        value: tx.value,
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
