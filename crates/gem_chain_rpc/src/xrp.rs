use async_trait::async_trait;
use hex;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use gem_xrp::rpc::{XRPClient, XRPMapper};
use primitives::{Asset, AssetBalance, AssetId, AssetType, Chain, Transaction};

const XRP_EPOCH_OFFSET_SECONDS: i64 = 946684800; // XRP epoch starts 2000-01-01

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
        let ledger = self.client.get_ledger_current().await?;
        Ok(ledger.ledger_current_index)
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
        let account = response.account_objects.first().ok_or("No account objects found for token_id")?;

        // Decode currency from hex, filter out null bytes, then convert to String
        let currency_bytes: Vec<u8> = hex::decode(&account.low_limit.currency)?.into_iter().filter(|&b| b != 0).collect();
        let symbol = String::from_utf8(currency_bytes).map_err(|e| format!("Failed to convert currency bytes to string: {}", e))?;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            symbol.clone(),
            symbol.clone(),
            15,
            AssetType::TOKEN,
        ))
    }
}

#[async_trait]
impl ChainAssetsProvider for XRPProvider {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
