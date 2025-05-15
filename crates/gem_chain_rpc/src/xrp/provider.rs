use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetId, AssetType};

use super::client::XRPClient;
use super::mapper::XRPMapper;

pub struct XRPProvider {
    client: XRPClient,
}

impl XRPProvider {
    pub fn new(client: XRPClient) -> Self {
        Self { client }
    }

    // Transaction mapping has been moved to XRPMapper
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

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block_transactions(block_number).await?;
        let block_timestamp = 946684800 + block.close_time;
        let transactions = block.transactions;

        let transactions = transactions
            .into_iter()
            .flat_map(|x| XRPMapper::map_transaction(self.get_chain(), x, block_number, block_timestamp))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for XRPProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_account_objects(token_id.clone()).await?;
        let account = response.account_objects.first().ok_or("No account objects found for token_id")?;

        let symbol = String::from_utf8(hex::decode(&account.low_limit.currency)?.into_iter().filter(|&b| b != 0).collect())
            .map_err(|_| "Failed to convert currency bytes to string")?;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            symbol.clone(),
            symbol.clone(),
            15,
            AssetType::TOKEN,
        ))
    }
}
