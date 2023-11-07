use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use primitives::{chain::Chain, Transaction, TransactionType, TransactionState, asset_id::AssetId};
use chrono::Utc;

use super::model::{Block, NodeInfo};
use reqwest_middleware::ClientWithMiddleware;

pub struct BNBChainClient {
    url: String,
    api_url: String,
    
    client: ClientWithMiddleware,
}

impl BNBChainClient {
    pub fn new(client: ClientWithMiddleware, url: String, api_url: String) -> Self {
        Self {
            url,
            api_url,
            client,
        }
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction) -> Option<primitives::Transaction> {
        if transaction.r#type != "TRANSFER" || transaction.from_addr.is_none()  {
            return None;
        }
        let token_id = if transaction.asset == Some(self.get_chain().as_denom().to_string()) { None } else { transaction.asset };
        let asset_id = AssetId{chain: self.get_chain(), token_id};

        let transaction = primitives::Transaction::new(
            transaction.hash,
            asset_id,
            transaction.from_addr.unwrap_or_default(),
            transaction.to_addr.unwrap_or_default(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            transaction.block_height.to_string(),
            transaction.sequence.to_string(),
            transaction.fee.to_string(),
            self.get_chain().as_asset_id(),
            format!("{:?}", transaction.amount.unwrap_or_default()),
            transaction.memo.into(),
            Utc::now()
        );
        Some(transaction)
    }
}

#[async_trait]
impl ChainProvider for BNBChainClient {

    fn get_chain(&self) -> Chain {
        Chain::Binance
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v1/node-info", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<NodeInfo>()
            .await?;

        Ok(response.sync_info.latest_block_height.into())
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/bc/api/v1/blocks/{}/txs", self.api_url, block);
        let transactions = self.client
            .get(url)
            .send()
            .await?
            .json::<Block>()
            .await?
            .txs.into_iter()
            .flat_map(|x| self.map_transaction(x))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}