use std::error::Error;

use primitives::{chain::Chain, transaction_utxo::TransactionInput, TransactionDirection, TransactionType};
use chrono::Utc;

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

    pub async fn get_tip_number(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
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

impl CardanoClient {
    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_tip_number().await?)
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<primitives::Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
