use async_trait::async_trait;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use gem_xrp::rpc::{XRPClient, XRPMapper};
use primitives::{Asset, AssetBalance, Chain, Transaction};

pub struct XRPProvider {
    client: XRPClient,
}

impl XRPProvider {
    pub fn new(client: XRPClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for XRPProvider {
    fn get_chain(&self) -> Chain {
        Chain::Xrp
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_ledger_current().await?.ledger_current_index)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block_transactions(block_number).await?;
        Ok(XRPMapper::map_block_transactions(self.get_chain(), block))
    }
}

#[async_trait]
impl ChainTokenDataProvider for XRPProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_account_objects(token_id.clone()).await?;
        XRPMapper::map_token_data(self.get_chain(), response.account_objects)
    }
}

#[async_trait]
impl ChainAssetsProvider for XRPProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let assets = self.client.get_account_objects(address.clone()).await?;
        Ok(XRPMapper::map_token_balances(self.get_chain(), assets.account_objects))
    }
}

#[async_trait]
impl ChainTransactionsProvider for XRPProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_account_transactions(address.clone(), 20).await?;
        Ok(XRPMapper::map_account_transactions(self.get_chain(), block))
    }
}

#[async_trait]
impl ChainStakeProvider for XRPProvider { }
