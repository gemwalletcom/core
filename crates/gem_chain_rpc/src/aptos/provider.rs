use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::AssetBalance;
use primitives::{chain::Chain, Asset, AssetId, AssetType};

use super::client::AptosClient;
use super::mapper::AptosMapper;
use gem_aptos::model::ResourceCoinInfo;

pub struct AptosProvider {
    client: AptosClient,
}

impl AptosProvider {
    pub fn new(client: AptosClient) -> Self {
        Self { client }
    }

    // Transaction mapping has been moved to AptosMapper
}

#[async_trait]
impl ChainBlockProvider for AptosProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let ledger = self.client.get_ledger().await?;
        Ok(ledger.block_height.parse::<i64>().unwrap_or_default())
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_block_transactions(block_number).await?.transactions;
        let transactions = transactions
            .into_iter()
            .flat_map(|x| AptosMapper::map_transaction(self.get_chain(), x, block_number))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AptosProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = token_id.split("::").collect();
        let address = parts.first().ok_or("Invalid token id")?;
        let resource = format!("0x1::coin::CoinInfo<{}>", token_id);
        let coin_info = self.client.get_resource::<ResourceCoinInfo>(address.to_string(), resource).await?.data;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            coin_info.name,
            coin_info.symbol,
            coin_info.decimals,
            AssetType::TOKEN,
        ))
    }
}

#[async_trait]
impl ChainAssetsProvider for AptosProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
