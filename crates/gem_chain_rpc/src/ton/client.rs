use std::error::Error;

use primitives::{chain::Chain, Asset, AssetId, AssetType};

use reqwest_middleware::ClientWithMiddleware;

use super::mapper::TonMapper;
use super::model::{Blocks, Chainhead, JettonInfo, Shards, Transactions};

pub struct TonClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TonClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    // Transaction mapping methods have been moved to TonMapper

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/masterchain-head", self.url);
        let response = self.client.get(url).send().await?.json::<Chainhead>().await?;
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

    pub async fn get_transactions_in_all_blocks(&self, block_id: String) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/masterchain/{}/transactions", self.url, block_id);
        let response = self.client.get(url).send().await?.json::<Transactions>().await?;

        Ok(response)
    }

    pub async fn get_block_transactions(&self, block_id: String) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/blocks/{}/transactions", self.url, block_id);
        let response = self.client.get(url).send().await?.json::<Transactions>().await?;

        Ok(response)
    }

    pub async fn get_token_info(&self, token_id: String) -> Result<JettonInfo, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/jettons/{}", self.url, token_id);
        Ok(self.client.get(url).send().await?.json::<JettonInfo>().await?)
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let chainhead = self.get_master_head().await?;
        Ok(chainhead.seqno)
    }

    pub async fn get_transactions(&self, block: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
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
            .flat_map(|x| TonMapper::map_transaction(self.get_chain(), x))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let token_info = self.get_token_info(token_id.clone()).await?;
        let decimals = token_info.metadata.decimals.parse::<i32>().map_err(|_| "Invalid decimals")?;
        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            token_info.metadata.name,
            token_info.metadata.symbol,
            decimals,
            AssetType::JETTON,
        ))
    }
}
