use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, StakeValidator, Transaction};

use gem_client::Client;
use gem_sui::rpc::client::SuiClient;

pub struct SuiProvider<C: Client> {
    client: SuiClient<C>,
}

impl<C: Client> SuiProvider<C> {
    pub fn new(client: SuiClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for SuiProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_latest_block().await? as i64)
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for SuiProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for SuiProvider<C> {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let balances = self.client.get_all_balances(address).await?;

        let asset_balances = balances
            .into_iter()
            .flat_map(|x| {
                let asset_id = if x.coin_type == self.client.get_chain().as_denom().unwrap_or_default() {
                    None
                } else {
                    Some(AssetId::from_token(self.client.get_chain(), &x.coin_type))
                };

                asset_id.map(|asset_id| AssetBalance::new(asset_id, x.total_balance.to_string()))
            })
            .collect::<Vec<_>>();

        Ok(asset_balances)
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for SuiProvider<C> {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for SuiProvider<C> {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        let validators = self.client.get_validators().await?;
        let stake_validators = validators.apys
            .into_iter()
            .map(|v| StakeValidator::new(v.address.clone(), v.address))
            .collect();
        Ok(stake_validators)
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let validator_set = self.client.get_validators().await?;
        let max_apy = validator_set.apys.into_iter().map(|v| v.apy).fold(0.0, f64::max);
        Ok(max_apy * 100.0)
    }
}
