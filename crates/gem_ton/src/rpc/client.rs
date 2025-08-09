use std::error::Error;

use primitives::{chain::Chain, Asset, AssetId, AssetType};

use reqwest_middleware::ClientWithMiddleware;

use crate::rpc::model::{JettonBalances, JettonInfo, TonApiError};

use super::model::{Blocks, Chainhead, Shards, Transactions};

pub struct TonClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TonClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
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

    pub async fn get_transactions(&self, block_id: String) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .get(format!("{}/v2/blockchain/masterchain/{}/transactions", self.url, block_id))
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        match status.as_u16() {
            200 => {
                // Success - parse as Transactions
                match serde_json::from_str::<Transactions>(&response_text) {
                    Ok(transactions) => Ok(transactions),
                    Err(e) => Err(format!("Failed to parse successful TON API response: {e}").into()),
                }
            }
            404 => {
                // Not Found - parse as error and propagate
                match serde_json::from_str::<TonApiError>(&response_text) {
                    Ok(error_response) => Err(error_response.error.into()),
                    Err(_) => Err("Block not found".into()),
                }
            }
            _ => {
                // Other HTTP errors
                Err(format!("TON API error: HTTP {status} - {response_text}").into())
            }
        }
    }

    pub async fn get_transactions_by_address(&self, address: String, limit: usize) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/v2/blockchain/accounts/{}/transactions?sort_order=desc&limit={}", self.url, address, limit);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account_jettons(&self, address: String) -> Result<JettonBalances, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(format!("{}/v2/accounts/{}/jettons", self.url, address))
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_token_info(&self, token_id: String) -> Result<JettonInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(format!("{}/v2/jettons/{}", self.url, token_id)).send().await?.json().await?)
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_master_head().await?.seqno)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let token_info = self.get_token_info(token_id.clone()).await?;
        let decimals = token_info.metadata.decimals as i32;
        Ok(Asset::new(
            AssetId::from_token(Chain::Ton, &token_id),
            token_info.metadata.name,
            token_info.metadata.symbol,
            decimals,
            AssetType::JETTON,
        ))
    }
}
