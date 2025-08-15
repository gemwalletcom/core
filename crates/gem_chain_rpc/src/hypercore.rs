use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, StakeValidator, Transaction};

use gem_client::Client;
use gem_hypercore::rpc::client::HyperCoreClient;

pub struct HyperCoreProvider<C: Client> {
    client: HyperCoreClient<C>,
}

impl<C: Client> HyperCoreProvider<C> {
    pub fn new(client: HyperCoreClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for HyperCoreProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::HyperCore
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(0)
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for HyperCoreProvider<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!("HyperCoreProvider::get_token_data")
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for HyperCoreProvider<C> {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for HyperCoreProvider<C> {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for HyperCoreProvider<C> {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get_staking_validators()
            .await?
            .into_iter()
            .filter(|x| x.is_active)
            .map(|x| StakeValidator::new(x.validator, x.name))
            .collect())
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        self.client.get_staking_apy().await
    }
}
