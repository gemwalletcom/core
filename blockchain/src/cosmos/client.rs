use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use primitives::{chain::Chain, TransactionType, TransactionState};
use reqwest_middleware::ClientWithMiddleware;
use super::model::BlockResponse;
use base64::{engine::general_purpose, Engine as _};
use sha2::{Sha256, Digest};
use hex;

const MESSAGE_SEND: &str = "/cosmos.bank.v1beta1.MsgSend";

pub struct CosmosClient {
    chain: Chain,
    url: String,
    client: ClientWithMiddleware,
}

impl CosmosClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self {
            chain,
            url,
            client,
        }
    }

    pub fn map_transaction(&self, block_number: i64, transaction: String) -> Option<primitives::Transaction> {
        let bytes = general_purpose::STANDARD.decode(transaction).ok()?;
        let tx: cosmos_sdk_proto::cosmos::tx::v1beta1::Tx = cosmos_sdk_proto::prost::Message::decode(&*bytes).unwrap();
        match tx.body {
            Some(body) => {
                for message in body.messages {
                    let hash = hex::encode(Sha256::digest(bytes.clone()));
                    let sequence = tx.auth_info.clone().unwrap().signer_infos.first()?.sequence;
                    let default_denom = self.chain.as_denom();
                    match message.type_url.as_str() {
                        MESSAGE_SEND => {
                            let message_send: cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend = cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;
                            let fee = tx.auth_info.clone().unwrap().fee?.amount.into_iter().filter(|x| x.denom ==default_denom).collect::<Vec<_>>();
                            let coins = message_send.amount.clone().into_iter().filter(|x| x.denom == default_denom).collect::<Vec<_>>();
                            let value = coins.first()?;
                            let fee = fee.first()?.amount.clone();
                            let memo = body.memo.clone();
                            let asset_id = self.get_chain().as_asset_id();
                            let transaction = primitives::Transaction::new(
                                hash,
                                asset_id.clone(),
                                message_send.from_address,
                                message_send.to_address,
                                None,
                                TransactionType::Transfer,
                                TransactionState::Confirmed,
                                block_number.to_string(),
                                sequence.to_string(),
                                fee,
                                asset_id.clone(),
                                value.clone().amount,
                                Some(memo),
                                None,
                                Utc::now(),
                            );
                            return Some(transaction)
                        },
                        _ => {
                            //println!("message.type_url: {:?}", message.type_url);
                        }
                    }
                }

            },
            None => {
                //println!("error: {:?}", e);
            }
        }
       None
   }

    pub async fn get_block(&self, block: &str) -> Result<BlockResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/cosmos/base/tendermint/v1beta1/blocks/{}", self.url, block);
        let block = self.client
            .get(url)
            .send()
            .await?
            .json::<BlockResponse>()
            .await?;
        Ok(block)
    }

}

#[async_trait]
impl ChainProvider for CosmosClient {

    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.get_block("latest").await?;
        let block_number = block.block.header.height.parse::<i64>()?;
        return Ok(block_number)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let response = self.get_block(block.to_string().as_str()).await?;
        let transactions = response.block.data.txs.into_iter().flat_map(|x| {
            self.map_transaction(block, x)
        }).collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}