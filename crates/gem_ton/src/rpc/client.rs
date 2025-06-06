use std::error::Error;

use primitives::{chain::Chain, Asset, AssetId, AssetType};

use reqwest_middleware::ClientWithMiddleware;

use super::mapper::TonMapper;
use super::model::{Blocks, Chainhead, JettonInfo, Shards, Transactions};
use super::model_rpc::AssetsBalances;

pub struct TonClient {
    url: String,
    client: ClientWithMiddleware,
    rpc_url: String,
}

impl TonClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self {
            url,
            client,
            rpc_url: "https://toncenter.com".to_string(), //TODO: Add to config
        }
    }

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/blockchain/masterchain-head", self.url))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_shards(&self, sequence: i64) -> Result<Shards, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/blockchain/masterchain/{}/shards", self.url, sequence))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_blocks(&self, sequence: i64) -> Result<Blocks, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/blockchain/masterchain/{}/blocks", self.url, sequence))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_transactions_in_all_blocks(&self, block_id: String) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/blockchain/masterchain/{}/transactions", self.url, block_id))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_block_transactions(&self, block_id: String) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/blockchain/blocks/{}/transactions", self.url, block_id))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_token_info(&self, token_id: String) -> Result<JettonInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(format!("{}/v2/jettons/{}", self.url, token_id)).send().await?.json().await?)
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_master_head().await?.seqno)
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
        let decimals = token_info.metadata.decimals as i32;
        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            token_info.metadata.name,
            token_info.metadata.symbol,
            decimals,
            AssetType::JETTON,
        ))
    }

    pub async fn get_assets_balances(
        &self,
        owner_address: String,
        exclude_zero_balance: bool,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<AssetsBalances, Box<dyn Error + Send + Sync>> {
        let limit_val = limit.unwrap_or(1000);
        let offset_val = offset.unwrap_or(0);

        let url = format!(
            "{}/api/v3/jetton/wallets?owner_address={}&exclude_zero_balance={}&limit={}&offset={}",
            self.rpc_url, owner_address, exclude_zero_balance, limit_val, offset_val
        );

        Ok(self.client.get(url).send().await?.json().await?)
    }
}
