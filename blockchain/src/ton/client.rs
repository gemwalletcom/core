use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use ns_address_codec::{ton::TonCodec, codec::Codec};
use primitives::{chain::Chain, TransactionType, TransactionState, TransactionDirection, asset_id::AssetId};

use reqwest_middleware::ClientWithMiddleware;

use super::model::{Transactions, NodeResultBlockType, NodeResult, NodeResultShardsType};

pub struct TonClient {
    url: String,
    api_url: String,
    client: ClientWithMiddleware,
}

impl TonClient {
    pub fn new(client: ClientWithMiddleware, url: String, api_url: String) -> Self {
        Self {
            url,
            api_url,
            client,
        }
    }

    pub fn map_transaction(&self, transaction: super::model::Transaction,) -> Option<primitives::Transaction> {
        // system transfer
        if transaction.transaction_type == "TransOrd" && transaction.total_fees > 0 && transaction.out_msgs.len() == 1 {
            let out_message = transaction.out_msgs.first().unwrap();
            let asset_id = AssetId::from_chain(self.get_chain());
            let from = TonCodec::encode(out_message.source.clone().address.as_bytes().to_vec());
            let to = TonCodec::encode(out_message.destination.clone().unwrap().address.as_bytes().to_vec());
            let state = if transaction.success && out_message.bounced == false { TransactionState::Confirmed } else { TransactionState::Failed };
            //TODO: Implement memo
            let memo: Option<String> = None; //out_message.decoded_body.clone().text;

            let transaction = primitives::Transaction{
                id: "".to_string(),
                hash: transaction.hash,
                asset_id: asset_id.clone(),
                from,
                to,
                contract: None,
                transaction_type: TransactionType::Transfer,
                state,
                block_number: 0,
                sequence: 0,
                fee: transaction.total_fees.to_string(),
                fee_asset_id: asset_id,
                value: out_message.value.to_string(),
                memo,
                direction: TransactionDirection::SelfTransfer,
                created_at: Utc::now().naive_utc(),
                updated_at: Utc::now().naive_utc(),
            };
            return Some(transaction)
        }
        return None
    }
}

#[async_trait]
impl ChainProvider for TonClient {

    fn get_chain(&self) -> Chain {
        Chain::Ton
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/v2/getMasterchainInfo", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<NodeResult<NodeResultBlockType>>()
            .await?;
        let url = format!("{}/api/v2/shards?seqno={}", self.url, response.result.last.seqno);

        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<NodeResult<NodeResultShardsType>>()
            .await?;

        Ok(response.result.shards.first().unwrap().seqno)
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        //TODO: Specify correct shard
        let reference = format!("(0,8000000000000000,{:?})", block);
        let url = format!("{}/v2/blockchain/blocks/{}/transactions", self.api_url, reference);

        let transactions = self.client
            .get(url.clone())
            .send()
            .await?
            .json::<Transactions>()
            .await?
            .transactions.into_iter()
            .flat_map(|x| self.map_transaction(x))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}