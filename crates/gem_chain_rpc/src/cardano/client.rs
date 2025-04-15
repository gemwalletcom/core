use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::Utc;
use primitives::{chain::Chain, transaction_utxo::TransactionInput, Asset, TransactionDirection, TransactionType};

use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, Blocks, Data, Transaction};

pub struct CardanoClient {
    chain: Chain,
    client: ClientWithMiddleware,
    url: String,
}

impl CardanoClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self {
            chain: Chain::Cardano,
            client,
            url,
        }
    }

    async fn get_tip_number(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "{ cardano { tip { number } } }"
        });
        let response: serde_json::Value = self.client.post(self.url.as_str()).json(&json).send().await?.json().await?;
        response["data"]["cardano"]["tip"]["number"]
            .as_i64()
            .ok_or_else(|| "Could not fetch tip number".into())
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "query GetBlockByNumber($blockNumber: Int!) { blocks(where: { number: { _eq: $blockNumber } }) { number hash forgedAt transactions { hash inputs { address value } outputs { address value } fee } } }",
            "variables": {
                "blockNumber": block_number
            },
            "operationName": "GetBlockByNumber"
        });
        self.client
            .post(self.url.as_str())
            .json(&json)
            .send()
            .await?
            .json::<Data<Blocks>>()
            .await?
            .data
            .blocks
            .first()
            .cloned()
            .ok_or_else(|| "Block not found".into())
    }

    pub fn map_transaction(chain: Chain, block: &Block, transaction: &Transaction) -> Option<primitives::Transaction> {
        let inputs: Vec<TransactionInput> = transaction
            .inputs
            .iter()
            .map(|x| TransactionInput {
                address: x.address.clone(),
                value: x.value.clone(),
            })
            .collect();

        let outputs: Vec<TransactionInput> = transaction
            .outputs
            .iter()
            .map(|x| TransactionInput {
                address: x.address.clone(),
                value: x.value.clone(),
            })
            .collect();

        if inputs.is_empty() || outputs.is_empty() {
            return None;
        }

        let transaction = primitives::Transaction::new_with_utxo(
            transaction.hash.clone(),
            chain.as_asset_id(),
            None,
            None,
            None,
            TransactionType::Transfer,
            primitives::TransactionState::Confirmed,
            block.number.to_string(),
            0.to_string(),
            transaction.fee.clone(),
            chain.as_asset_id(),
            "0".to_string(),
            None,
            TransactionDirection::SelfTransfer,
            inputs.into(),
            outputs.into(),
            None,
            Utc::now(),
        );

        Some(transaction)
    }
}

#[async_trait]
impl ChainBlockProvider for CardanoClient {
    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_tip_number().await?)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions = block
            .transactions
            .clone()
            .into_iter()
            .flat_map(|x| Self::map_transaction(self.chain, &block, &x))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for CardanoClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
