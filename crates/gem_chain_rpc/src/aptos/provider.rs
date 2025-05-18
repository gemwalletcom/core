use std::error::Error;

use async_trait::async_trait;
use gem_aptos::{CoinInfo, ResourceData, APTOS_NATIVE_COIN, COIN_INFO, COIN_STORE};
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, AssetType, Transaction};

use super::{client::AptosClient, mapper::AptosMapper};
use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};

#[derive(Clone)]
pub struct AptosProvider {
    client: AptosClient,
}

impl AptosProvider {
    pub fn new(client: AptosClient) -> Self {
        Self { client }
    }
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

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_block_transactions(block_number).await?.transactions;
        let transactions = transactions
            .into_iter()
            .flat_map(|x| AptosMapper::map_transaction(self.get_chain(), x, block_number))
            .collect::<Vec<_>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AptosProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = token_id.split("::").collect();
        let address = parts.first().ok_or("Invalid token id")?;
        let resource = format!("{}<{}>", COIN_INFO, token_id);
        let coin_info = self
            .client
            .get_account_resource::<CoinInfo>(address.to_string(), &resource)
            .await?
            .unwrap()
            .data;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            coin_info.name,
            coin_info.symbol,
            coin_info.decimals.into(),
            AssetType::TOKEN,
        ))
    }
}

#[async_trait]
impl ChainAssetsProvider for AptosProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let resources = self.client.get_account_resources(&address).await?;
        let balances = resources
            .into_iter()
            .filter_map(|resource| {
                let token_type = resource
                    .resource_type
                    .strip_prefix(&format!("{}<", COIN_STORE))
                    .and_then(|s| s.strip_suffix('>'))?;

                if token_type == APTOS_NATIVE_COIN {
                    return None;
                };
                match resource.data {
                    ResourceData::CoinStore(coin_store) => Some(AssetBalance {
                        asset_id: AssetId::from_token(self.get_chain(), token_type),
                        balance: coin_store.coin.value,
                    }),
                    _ => None,
                }
            })
            .collect();

        Ok(balances)
    }
}
