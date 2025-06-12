use async_trait::async_trait;
use gem_xrp::{XRP_DEFAULT_ASSET_DECIMALS, XRP_EPOCH_OFFSET_SECONDS};
use number_formatter::BigNumberFormatter;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use gem_xrp::rpc::{XRPClient, XRPMapper};
use primitives::{Asset, AssetBalance, AssetId, AssetType, Chain, Transaction};

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
        let block_timestamp = XRP_EPOCH_OFFSET_SECONDS + block.close_time;
        let transactions = block.transactions;

        let transactions = transactions
            .into_iter()
            .flat_map(|x| XRPMapper::map_transaction(self.get_chain(), x, block_number, block_timestamp))
            .collect::<Vec<Transaction>>();
        Ok(transactions)
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
        let balances = assets
            .account_objects
            .into_iter()
            .filter(|x| x.high_limit.currency.len() > 3)
            .flat_map(|x| {
                let asset_id = AssetId::from_token(self.get_chain(), &x.high_limit.issuer);
                let value = BigNumberFormatter::value_from_amount(&x.balance.value, XRP_DEFAULT_ASSET_DECIMALS as u32)?;
                Some(AssetBalance::new(asset_id, value))
            })
            .collect();
        Ok(balances)
    }
}

#[async_trait]
impl ChainTransactionsProvider for XRPProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
