use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use ns_address_codec::{ton::TonCodec, codec::Codec};
use primitives::{chain::Chain, TransactionType, TransactionState};

use reqwest::Url;
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Transaction, JSONResult, Chainhead, Shards, ShortTransactions};

pub struct TonClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TonClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self {
            url,
            client,
        }
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction) -> Option<primitives::Transaction> {
        // system transfer
        if transaction.fee != "0" && transaction.out_msgs.len() == 1 {
            let out_message = transaction.out_msgs.first().unwrap();
            let asset_id = self.get_chain().as_asset_id();
            let from = out_message.clone().source; 
            let to = out_message.clone().destination.unwrap_or_default(); 
            let state = TransactionState::Confirmed;
            //TODO: Implement memo
            let memo: Option<String> = None; //out_message.decoded_body.clone().text;

            let transaction = primitives::Transaction::new(
                transaction.transaction_id.hash,
                asset_id.clone(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                0.to_string(),
                0.to_string(),
                transaction.fee,
                asset_id,
                out_message.value.to_string(),
                memo,
                None,
                Utc::now()
            );
            return Some(transaction)
        }
        None
    }

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/getMasterchainInfo", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<JSONResult<Chainhead>>()
            .await?;
        Ok(response.result)
    }

    pub async fn get_shards(&self, sequence: i64) -> Result<Shards, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/shards?seqno={}", self.url, sequence);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<JSONResult<Shards>>()
            .await?;

        Ok(response.result)
    }

    pub async fn get_transactions(&self, workchain: i64, shard: i64, sequence: i64) -> Result<ShortTransactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/getBlockTransactions?workchain={}&shard={}&seqno={}&count=50", self.url, workchain, shard, sequence);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<JSONResult<ShortTransactions>>()
            .await?;

        Ok(response.result)
    }
    
    pub async fn get_transaction(&self, address: String, _hash: String, lt: String) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        let url = Url::parse_with_params(format!("{}/api/v2/getTransactions", self.url).as_str(), &[
            ("address", address),
            ("lt", lt),
            ("limit", "1".to_string()),
        ]).unwrap();

        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<JSONResult<Vec<Transaction>>>()
            .await?;

        let transaction = response.result.first().ok_or("No transaction found")?;
        Ok(transaction.clone())
    }

}

#[async_trait]
impl ChainProvider for TonClient {

    fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let chainhead = self.get_master_head().await?;
        let shards = self.get_shards(chainhead.last.seqno).await?.shards;
        let result = shards.first().ok_or("No shards found")?.seqno;
        Ok(result)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_transactions(0, 8000000000000000, block).await?;
        let futures = transactions.transactions.into_iter().map(|transaction| { 
            let address = TonCodec::encode(transaction.account.as_bytes().to_vec());
            return self.get_transaction(address, transaction.hash, transaction.lt) 
        });
        let transactions = futures::future::join_all(futures).await.into_iter().filter_map(Result::ok).collect::<Vec<Transaction>>()
            .into_iter()
            .flat_map(|x| self.map_transaction(x))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}