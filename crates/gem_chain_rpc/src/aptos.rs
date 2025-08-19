use std::error::Error;

use async_trait::async_trait;
use gem_aptos::{
    constants::{APTOS_NATIVE_COIN, COIN_INFO, COIN_STORE},
    models::CoinInfo,
    rpc::client::AptosClient,
};
use gem_client::Client;
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, AssetType, Transaction};

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};

pub struct AptosProvider<C: Client> {
    client: AptosClient<C>,
}

impl<C: Client> AptosProvider<C> {
    pub fn new(client: AptosClient<C>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C: Client> ChainBlockProvider for AptosProvider<C> {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let ledger = self.client.get_ledger().await?;
        Ok(ledger.ledger_version as i64)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let _transactions = self.client.get_block_transactions(block_number).await?.transactions;
        // TODO: Implement transaction mapping from Aptos format to primitives::Transaction
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainTokenDataProvider for AptosProvider<C> {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = token_id.split("::").collect();
        let address = parts.first().ok_or("Invalid token id")?;
        let resource_type_str = format!("{COIN_INFO}<{token_id}>");
        let coin_info_resource = self
            .client
            .get_account_resource::<CoinInfo>(address.to_string(), &resource_type_str)
            .await?
            .ok_or_else(|| format!("CoinInfo resource not found for token_id: {token_id}"))?;

        let coin_info_data = coin_info_resource.data;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            coin_info_data.name,
            coin_info_data.symbol,
            coin_info_data.decimals.into(),
            AssetType::TOKEN,
        ))
    }
}

#[async_trait]
impl<C: Client> ChainAssetsProvider for AptosProvider<C> {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let resources = self.client.get_account_resources(&address).await?;
        let balances = resources
            .into_iter()
            .filter_map(|resource| {
                let token_type = resource
                    .type_field
                    .strip_prefix(&format!("{COIN_STORE}<"))
                    .and_then(|s| s.strip_suffix('>'))?;

                if token_type == APTOS_NATIVE_COIN {
                    return None;
                };
                if let Some(coin_data) = &resource.data.coin {
                    Some(AssetBalance::new(AssetId::from_token(self.get_chain(), token_type), coin_data.value.clone()))
                } else {
                    None
                }
            })
            .collect();

        Ok(balances)
    }
}

#[async_trait]
impl<C: Client> ChainTransactionsProvider for AptosProvider<C> {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let _transactions = self.client.get_transactions_by_address(address).await?;
        // TODO: Map transactions using the provider mapper pattern 
        Ok(vec![])
    }
}

#[async_trait]
impl<C: Client> ChainStakeProvider for AptosProvider<C> { }
