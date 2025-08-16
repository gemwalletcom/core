use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, Transaction};

use gem_ton::{
    rpc::{TonClient, TonMapper},
    TonAddress,
};

pub struct TonProvider {
    client: TonClient,
}

impl TonProvider {
    pub fn new(client: TonClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for TonProvider {
    fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_transactions(block_number.to_string()).await?.transactions;
        Ok(TonMapper::map_transactions(self.get_chain(), transactions))
    }
}

#[async_trait]
impl ChainTokenDataProvider for TonProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl ChainAssetsProvider for TonProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_account_jettons(address).await?;
        let balances = response
            .balances
            .into_iter()
            .filter_map(|x| {
                let ton_address = TonAddress::from_hex_str(&x.jetton.address).ok()?;
                let asset_id = AssetId::from_token(self.get_chain(), &ton_address.to_base64_url());
                Some(AssetBalance::new(asset_id, x.balance))
            })
            .collect();
        Ok(balances)
    }
}

#[async_trait]
impl ChainTransactionsProvider for TonProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_transactions_by_address(address, 20).await?.transactions;
        Ok(TonMapper::map_transactions(self.get_chain(), transactions))
    }
}

#[async_trait]
impl ChainStakeProvider for TonProvider { }
