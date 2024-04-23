use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use gem_ton::address::TonAddress;
use primitives::{chain::Chain, TransactionState, TransactionType};

use reqwest_middleware::ClientWithMiddleware;

use super::model::{Blocks, Chainhead, Shards, Transaction, Transactions};

pub struct TonClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TonClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub fn parse_address(address: &str) -> Option<String> {
        let address = TonAddress::from_hex_str(address).ok()?;
        Some(address.to_base64_url())
    }

    pub fn map_transaction(&self, transaction: Transaction) -> Option<primitives::Transaction> {
        if transaction.transaction_type == "TransOrd"
            && transaction.out_msgs.len() == 1
            && transaction.out_msgs.first()?.op_code.is_none()
        {
            let asset_id = self.get_chain().as_asset_id();
            let out_message = transaction.out_msgs.first()?;
            let from = Self::parse_address(&out_message.source.address)?;
            let to: String = match &out_message.destination {
                Some(destination) => Self::parse_address(&destination.address)?,
                None => "".into(),
            };
            let value = out_message.value.to_string();
            let state = if transaction.success {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };
            //TODO: Implement memo
            let memo: Option<String> = None; //out_message.decoded_body.clone().text;

            let transaction = primitives::Transaction::new(
                transaction.hash,
                asset_id.clone(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                transaction.block.to_string(),
                0.to_string(),
                transaction.total_fees.to_string(),
                asset_id,
                value,
                memo,
                None,
                Utc::now(),
            );
            return Some(transaction);
        }
        None
    }

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/masterchain-head", self.url);
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Chainhead>()
            .await?;
        Ok(response)
    }

    pub async fn get_shards(&self, sequence: i64) -> Result<Shards, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/masterchain/{}/shards", self.url, sequence);
        let response = self.client.get(url).send().await?.json::<Shards>().await?;
        Ok(response)
    }

    pub async fn get_blocks(&self, sequence: i64) -> Result<Blocks, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/masterchain/{}/blocks", self.url, sequence);
        let response = self.client.get(url).send().await?.json::<Blocks>().await?;
        Ok(response)
    }

    pub async fn get_transactions_in_all_blocks(
        &self,
        block_id: String,
    ) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/v2/blockchain/masterchain/{}/transactions",
            self.url, block_id
        );
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Transactions>()
            .await?;

        Ok(response)
    }

    pub async fn get_block_transactions(
        &self,
        block_id: String,
    ) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/v2/blockchain/blocks/{}/transactions",
            self.url, block_id
        );
        let response = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Transactions>()
            .await?;

        Ok(response)
    }
}

#[async_trait]
impl ChainProvider for TonClient {
    fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let chainhead = self.get_master_head().await?;
        Ok(chainhead.seqno)
    }

    async fn get_transactions(
        &self,
        block: i64,
    ) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        // let shards = self.get_blocks(block).await?.blocks;

        // let futures = shards.into_iter().map(|shard| {
        //     return self.get_block_transactions(shard.seqno.to_string());
        // });

        // let transactions = futures::future::join_all(futures)
        //     .await
        //     .into_iter()
        //     .filter_map(Result::ok)
        //     .collect::<Vec<Transactions>>()
        //     .into_iter()
        //     .flat_map(|x| x.transactions)
        //     .flat_map(|x| self.map_transaction(x))
        //     .collect::<Vec<primitives::Transaction>>();

        let transactions = self
            .get_transactions_in_all_blocks(block.to_string())
            .await?
            .transactions
            .into_iter()
            .flat_map(|x| self.map_transaction(x))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}
