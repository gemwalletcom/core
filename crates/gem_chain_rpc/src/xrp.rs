use async_trait::async_trait;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use gem_client::Client;
use gem_xrp::rpc::{XRPClient, XRPMapper};
use primitives::{Asset, AssetBalance, Chain, Transaction};

pub struct XRPProvider<C: Client> {
    client: XRPClient<C>,
}

impl<C: Client> XRPProvider<C> {
    pub fn new(client: XRPClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for XRPProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Xrp
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_ledger_current().await?.ledger_current_index as i64)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block_transactions(block_number).await?;
        Ok(XRPMapper::map_block_transactions(self.get_chain(), block))
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for XRPProvider<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Err("Not implemented for new XRP provider".into())
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for XRPProvider<C> {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for XRPProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_account_transactions(address.clone(), 20).await?;
        Ok(XRPMapper::map_account_transactions(self.get_chain(), block))
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for XRPProvider<C> {}
