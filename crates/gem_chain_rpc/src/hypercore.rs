use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, Transaction};

use gem_hypercore::rpc::client::HyperCoreClient;

pub struct HyperCoreProvider {
    #[allow(dead_code)]
    client: HyperCoreClient,
}

impl HyperCoreProvider {
    pub fn new(client: HyperCoreClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for HyperCoreProvider {
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
impl ChainTokenDataProvider for HyperCoreProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!("HyperCoreProvider::get_token_data")
    }
}

#[async_trait]
impl ChainAssetsProvider for HyperCoreProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainTransactionsProvider for HyperCoreProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainStakeProvider for HyperCoreProvider {
    // Default implementation returns empty vector
}
