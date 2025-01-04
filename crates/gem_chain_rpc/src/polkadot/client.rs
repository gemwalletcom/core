use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;

use chrono::Utc;
use primitives::{Asset, Chain};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, BlockHeader, Extrinsic, ExtrinsicArguments, TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH, TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE};

pub struct PolkadotClient {
    url: String,
    client: ClientWithMiddleware,
}

impl PolkadotClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_block_header(&self, block: &str) -> Result<BlockHeader, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blocks/{}/header", self.url, block);
        Ok(self.client.get(url).send().await?.json::<BlockHeader>().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blocks/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<Block>().await?)
    }

    pub fn map_transaction(&self, block: Block, transaction: Extrinsic) -> Option<primitives::Transaction> {
        match transaction.method.method.as_str() {
            TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE | TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH => {
                let transfer = if let ExtrinsicArguments::Transfer(transfer) = &transaction.args {
                    transfer
                } else {
                    return None;
                };
                let from_address = transaction.signature?.signer.id.clone();
                let state = if transaction.success {
                    primitives::TransactionState::Confirmed
                } else {
                    primitives::TransactionState::Failed
                };
                Some(primitives::Transaction::new(
                    transaction.hash.clone(),
                    self.get_chain().as_asset_id(),
                    from_address,
                    transfer.dest.id.clone(),
                    None,
                    primitives::TransactionType::Transfer,
                    state,
                    block.number,
                    transaction.nonce.unwrap_or_default().clone(),
                    transaction.info.partial_fee.unwrap_or("0".to_string()),
                    self.get_chain().as_asset_id(),
                    transfer.value.clone(),
                    None,
                    None,
                    Utc::now(),
                ))
            }
            _ => None,
        }
    }
}

#[async_trait]
impl ChainBlockProvider for PolkadotClient {
    fn get_chain(&self) -> Chain {
        Chain::Stellar
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.get_block_header("head")
            .await?
            .number
            .parse()
            .map_err(|_| "Failed to parse block number".into())
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;

        let transactions = block
            .extrinsics
            .iter()
            .flat_map(|x| self.map_transaction(block.clone(), x.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for PolkadotClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
