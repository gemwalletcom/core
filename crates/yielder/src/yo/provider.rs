use std::{str::FromStr, sync::Arc};

use alloy_primitives::{Address, U256};
use async_trait::async_trait;
use gem_evm::jsonrpc::TransactionObject;
use primitives::AssetId;

use crate::provider::{
    Yield,
    YieldDepositRequest,
    YieldDetails,
    YieldDetailsRequest,
    YieldProvider,
    YieldTransaction,
    YieldWithdrawRequest,
};

use super::{
    client::YoGatewayApi,
    error::YieldError,
    vaults,
    YoVault,
    YO_PARTNER_ID_GEM,
};

#[derive(Clone)]
pub struct YoYieldProvider {
    vaults: Vec<YoVault>,
    gateway: Arc<dyn YoGatewayApi>,
}

impl YoYieldProvider {
    pub fn new(gateway: Arc<dyn YoGatewayApi>) -> Self {
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
}

#[async_trait]
impl YieldProvider for YoYieldProvider {
    fn protocol(&self) -> &'static str {
        "yo"
    }

    fn yields(&self, asset_id: &AssetId) -> Vec<Yield> {
        self.vaults
            .iter()
            .filter_map(|vault| {
                let vault_asset = vault.asset_id();
                if &vault_asset == asset_id {
                    Some(Yield::new(vault.name, vault_asset, self.protocol(), None))
                } else {
                    None
                }
            })
            .collect()
    }

    async fn deposit(&self, request: &YieldDepositRequest) -> Result<YieldTransaction, YieldError> {
        let vault = self.find_vault(&request.asset)?;
        let wallet = parse_address(&request.wallet_address)?;
        let receiver = match &request.receiver_address {
            Some(address) => parse_address(address)?,
            None => wallet,
        };
        let amount = parse_amount(&request.amount)?;
        let min_shares = parse_amount(request.min_shares.as_deref().unwrap_or("0"))?;
        let partner_id = request.partner_id.unwrap_or(YO_PARTNER_ID_GEM);

        let tx = self
            .gateway
            .build_deposit_transaction(wallet, vault.yo_token, amount, min_shares, receiver, partner_id);
        Ok(convert_transaction(vault, tx))
    }

    async fn withdraw(&self, request: &YieldWithdrawRequest) -> Result<YieldTransaction, YieldError> {
        let vault = self.find_vault(&request.asset)?;
        let wallet = parse_address(&request.wallet_address)?;
        let receiver = match &request.receiver_address {
            Some(address) => parse_address(address)?,
            None => wallet,
        };
        let shares = parse_amount(&request.shares)?;
        let min_assets = parse_amount(request.min_assets.as_deref().unwrap_or("0"))?;
        let partner_id = request.partner_id.unwrap_or(YO_PARTNER_ID_GEM);

        let tx = self
            .gateway
            .build_redeem_transaction(wallet, vault.yo_token, shares, min_assets, receiver, partner_id);
        Ok(convert_transaction(vault, tx))
    }

    async fn details(&self, request: &YieldDetailsRequest) -> Result<YieldDetails, YieldError> {
        let vault = self.find_vault(&request.asset)?;
        let owner = parse_address(&request.wallet_address)?;
        let mut details = YieldDetails::new(request.asset.clone(), self.protocol(), vault.yo_token, vault.asset_token);

        let share_balance = self.gateway.balance_of(vault.yo_token, owner).await?;
        details.share_balance = Some(share_balance.to_string());

        let asset_balance = self.gateway.balance_of(vault.asset_token, owner).await?;
        details.asset_balance = Some(asset_balance.to_string());

        Ok(details)
    }
}

fn parse_address(value: &str) -> Result<Address, YieldError> {
    Address::from_str(value).map_err(|err| YieldError::new(format!("invalid address {value}: {err}")))
}

fn parse_amount(value: &str) -> Result<U256, YieldError> {
    U256::from_str_radix(value, 10).map_err(|err| YieldError::new(format!("invalid amount {value}: {err}")))
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
