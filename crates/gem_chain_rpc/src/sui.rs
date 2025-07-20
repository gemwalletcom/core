use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, StakeValidator, Transaction};

use gem_sui::rpc::{client::SuiClient, mapper::SuiMapper};

pub struct SuiProvider {
    client: SuiClient,
}

impl SuiProvider {
    pub fn new(client: SuiClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for SuiProvider {
    fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get_transactions_by_block_number(block_number)
            .await?
            .data
            .into_iter()
            .flat_map(SuiMapper::map_transaction)
            .collect())
    }
}

#[async_trait]
impl ChainTokenDataProvider for SuiProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let metadata = self.client.get_coin_metadata(token_id.clone()).await?;
        Ok(SuiMapper::map_token(self.get_chain(), metadata))
    }
}

#[async_trait]
impl ChainAssetsProvider for SuiProvider {
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

                asset_id.map(|asset_id| AssetBalance {
                    asset_id,
                    balance: x.total_balance,
                })
            })
            .collect::<Vec<_>>();

        Ok(asset_balances)
    }
}

#[async_trait]
impl ChainTransactionsProvider for SuiProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get_transactions_by_address(address)
            .await?
            .data
            .into_iter()
            .flat_map(SuiMapper::map_transaction)
            .collect())
    }
}

#[async_trait]
impl ChainStakeProvider for SuiProvider {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        let validators = self.client.get_validators().await?;
        Ok(validators
            .apys
            .into_iter()
            .map(|v| StakeValidator {
                id: v.address.clone(),
                name: v.address,
            })
            .collect())
    }
}
