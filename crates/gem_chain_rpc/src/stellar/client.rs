use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;

use primitives::{Asset, Chain};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, Embedded, NodeStatus, Payment, TRANSACTION_TYPE_CREATE_ACCOUNT, TRANSACTION_TYPE_PAYMENT};

pub struct StellarClient {
    url: String,
    client: ClientWithMiddleware,
}

impl StellarClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_node_status(&self) -> Result<NodeStatus, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/", self.url);
        Ok(self.client.get(url).send().await?.json::<NodeStatus>().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/ledgers/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<Block>().await?)
    }

    pub async fn get_block_payments(&self, block_number: i64) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let query = [("limit", 200.to_string()), ("include_failed", "true".to_string())];
        let url = format!("{}/ledgers/{}/payments", self.url, block_number);
        Ok(self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .json::<Embedded<Payment>>()
            .await?
            ._embedded
            .records)
    }

    pub fn map_transaction(&self, block: Block, transaction: Payment) -> Option<primitives::Transaction> {
        match transaction.payment_type.as_str() {
            TRANSACTION_TYPE_PAYMENT | TRANSACTION_TYPE_CREATE_ACCOUNT => {
                if transaction.clone().asset_type.unwrap_or_default() == "native"
                    || transaction.clone().payment_type.as_str() == TRANSACTION_TYPE_CREATE_ACCOUNT
                {
                    return Some(primitives::Transaction::new(
                        transaction.clone().transaction_hash,
                        self.get_chain().as_asset_id(),
                        transaction.clone().from.unwrap_or_default(),
                        transaction.clone().to.unwrap_or_default(),
                        None,
                        primitives::TransactionType::Transfer,
                        primitives::TransactionState::Confirmed,
                        block.sequence.to_string(),
                        0.to_string(),
                        block.base_fee_in_stroops.to_string(), // TODO: Calculate from block/transaction
                        self.get_chain().as_asset_id(),
                        transaction.get_value().unwrap_or("0".to_string()).to_string(),
                        transaction.clone().get_memo(),
                        None,
                        block.closed_at.parse().unwrap_or_default(),
                    ));
                }

                None
            }
            _ => None,
        }
    }
}

#[async_trait]
impl ChainBlockProvider for StellarClient {
    fn get_chain(&self) -> Chain {
        Chain::Stellar
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_node_status().await?.history_latest_ledger)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions = self
            .get_block_payments(block_number)
            .await?
            .iter()
            .flat_map(|x| self.map_transaction(block.clone(), x.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for StellarClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
