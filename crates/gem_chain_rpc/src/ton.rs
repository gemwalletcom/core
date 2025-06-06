use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, Transaction};

use gem_ton::{address::TonAddress, rpc::TonClient};

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
        self.client.get_transactions(block_number).await
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
        let response = self.client.get_assets_balances(address, true, None, None).await?;
        let balances = response
            .jetton_wallets
            .into_iter()
            .flat_map(|x| {
                let ton_address = TonAddress::from_hex_str(&x.jetton).ok()?;
                let asset_id = AssetId::from_token(self.get_chain(), &ton_address.to_base64_url());
                Some(AssetBalance::new(asset_id, x.balance))
            })
            .collect();
        Ok(balances)
    }
}
