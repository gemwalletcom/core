use std::{str::FromStr, sync::Arc};

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use gem_evm::jsonrpc::TransactionObject;
use primitives::AssetId;
use tokio::try_join;

use crate::provider::{Yield, YieldDetailsRequest, YieldPosition, YieldProvider, YieldProviderClient, YieldTransaction};

use super::{YO_PARTNER_ID_GEM, YoVault, client::YoProvider, error::YieldError, vaults};

const SECONDS_PER_YEAR: f64 = 31_536_000.0;
const APY_LOOKBACK_SECONDS: u64 = 7 * 24 * 60 * 60;

#[derive(Clone)]
pub struct YoYieldProvider {
    vaults: Vec<YoVault>,
    gateway: Arc<dyn YoProvider>,
}

impl YoYieldProvider {
    pub fn new(gateway: Arc<dyn YoProvider>) -> Self {
        Self {
            vaults: vaults().to_vec(),
            gateway,
        }
    }

    fn find_vault(&self, asset_id: &AssetId) -> Result<YoVault, YieldError> {
        self.vaults
            .iter()
            .copied()
            .find(|vault| vault.asset_id() == *asset_id)
            .ok_or_else(|| YieldError::new(format!("unsupported asset {}", asset_id)))
    }

    async fn performance_apy(&self, vault: YoVault) -> Result<Option<f64>, YieldError> {
        let latest_block = self.gateway.latest_block_number().await?;
        let latest_timestamp = self.gateway.block_timestamp(latest_block).await?;
        let target_timestamp = latest_timestamp.saturating_sub(APY_LOOKBACK_SECONDS);
        let lookback_block = self.find_block_before(target_timestamp, latest_block).await?;
        let (latest_price, lookback_price) = try_join!(self.share_price_at_block(vault, latest_block), self.share_price_at_block(vault, lookback_block))?;
        let lookback_timestamp = self.gateway.block_timestamp(lookback_block).await?;
        let elapsed = latest_timestamp.saturating_sub(lookback_timestamp);
        Ok(annualize_growth(latest_price, lookback_price, elapsed))
    }

    async fn share_price_at_block(&self, vault: YoVault, block_number: u64) -> Result<U256, YieldError> {
        let one_share = U256::from(10u64).pow(U256::from(vault.asset_decimals));
        self.gateway.convert_to_assets_at_block(vault.yo_token, one_share, block_number).await
    }

    async fn find_block_before(&self, target_timestamp: u64, latest_block: u64) -> Result<u64, YieldError> {
        let mut low = 0;
        let mut high = latest_block;
        let mut candidate = latest_block;

        while low <= high {
            let mid = (low + high) / 2;
            let mid_timestamp = self.gateway.block_timestamp(mid).await?;

            if mid_timestamp > target_timestamp {
                if mid == 0 {
                    candidate = 0;
                    break;
                }
                high = mid - 1;
            } else {
                candidate = mid;
                low = mid + 1;
            }
        }

        Ok(candidate)
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
            let apy = self.performance_apy(vault).await?;
            results.push(Yield::new(vault.name, vault.asset_id(), self.provider(), apy));
        }

        Ok(results)
    }

    async fn deposit(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let vault = self.find_vault(asset_id)?;
        let wallet = parse_address(wallet_address)?;
        let receiver = wallet;
        let amount = parse_value(value)?;
        let min_shares = U256::from(0);
        let partner_id = YO_PARTNER_ID_GEM;

        let tx = self
            .gateway
            .build_deposit_transaction(wallet, vault.yo_token, amount, min_shares, receiver, partner_id);
        Ok(convert_transaction(vault, tx))
    }

    async fn withdraw(&self, asset_id: &AssetId, wallet_address: &str, value: &str) -> Result<YieldTransaction, YieldError> {
        let vault = self.find_vault(asset_id)?;
        let wallet = parse_address(wallet_address)?;
        let receiver = wallet;
        let shares = parse_value(value)?;
        let min_assets = U256::from(0);
        let partner_id = YO_PARTNER_ID_GEM;

        let tx = self
            .gateway
            .build_redeem_transaction(wallet, vault.yo_token, shares, min_assets, receiver, partner_id);
        Ok(convert_transaction(vault, tx))
    }

    async fn positions(&self, request: &YieldDetailsRequest) -> Result<YieldPosition, YieldError> {
        let vault = self.find_vault(&request.asset_id)?;
        let owner = parse_address(&request.wallet_address)?;
        let mut details = YieldPosition::new(request.asset_id.clone(), self.provider(), vault.yo_token, vault.asset_token);

        let share_balance = self.gateway.balance_of(vault.yo_token, owner).await?;
        details.vault_balance_value = Some(share_balance.to_string());

        let asset_balance = self.gateway.balance_of(vault.asset_token, owner).await?;
        details.asset_balance_value = Some(asset_balance.to_string());

        details.apy = self.performance_apy(vault).await?;

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
