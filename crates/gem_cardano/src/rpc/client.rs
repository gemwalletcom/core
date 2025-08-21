use std::error::Error;

use chain_traits::{ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::Client;
use primitives::chain::Chain;

use super::model::{Block, Blocks, Data};
use crate::models::{CardanoBalanceResponse, CardanoBlockData, CardanoGenesisData, CardanoTransactionBroadcast, CardanoUTXO, CardanoUTXOS};
use primitives::graphql::GraphqlData;

#[derive(Debug)]
pub struct CardanoClient<C: Client> {
    client: C,
    chain: Chain,
}

impl<C: Client> CardanoClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Cardano }
    }

    pub async fn get_tip_number(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "{ cardano { tip { number } } }"
        });
        let response: Data<CardanoBlockData> = self.client.post("/", &json, None).await?;
        Ok(response.data.cardano.tip.number as i64)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "query": "query GetBlockByNumber($blockNumber: Int!) { blocks(where: { number: { _eq: $blockNumber } }) { number hash forgedAt transactions { hash inputs { address value } outputs { address value } fee } } }",
            "variables": {
                "blockNumber": block_number
            },
            "operationName": "GetBlockByNumber"
        });
        let response: Data<Blocks> = self.client.post("/", &json, None).await?;
        response.data.blocks.first().cloned().ok_or_else(|| "Block not found".into())
    }

    pub async fn get_balance(&self, address: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "GetBalance",
            "variables": {"address": address},
            "query": "query GetBalance($address: String!) { utxos: utxos_aggregate(where: { address: { _eq: $address }  } ) { aggregate { sum { value } } } }"
        });
        let response: GraphqlData<CardanoBalanceResponse> = self.client.post("/", &json, None).await?;

        if let Some(errors) = response.errors {
            if let Some(error) = errors.first() {
                return Err(error.message.clone().into());
            }
        }

        if let Some(data) = response.data {
            Ok(data.utxos.aggregate.sum.value.unwrap_or_else(|| "0".to_string()))
        } else {
            Ok("0".to_string())
        }
    }

    pub async fn get_utxos(&self, address: &str) -> Result<Vec<CardanoUTXO>, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "UtxoSetForAddress",
            "variables": {"address": address},
            "query": "query UtxoSetForAddress($address: String!) { utxos(order_by: { value: desc } , where: { address: { _eq: $address }  } ) { address value txHash index tokens { quantity asset { fingerprint policyId assetName } } } }"
        });
        let response: Data<CardanoUTXOS<Vec<CardanoUTXO>>> = self.client.post("/", &json, None).await?;
        Ok(response.data.utxos)
    }

    pub async fn get_network_magic(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "GetNetworkMagic",
            "variables": {},
            "query": "query GetNetworkMagic { genesis { shelley { networkMagic } } }"
        });
        let response: Data<CardanoGenesisData> = self.client.post("/", &json, None).await?;
        Ok(response.data.genesis.shelley.network_magic.to_string())
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let json = serde_json::json!({
            "operationName": "SubmitTransaction",
            "variables": {"transaction": data},
            "query": "mutation SubmitTransaction($transaction: String!) { submitTransaction(transaction: $transaction) { hash } }"
        });
        let response: GraphqlData<CardanoTransactionBroadcast> = self.client.post("/", &json, None).await?;

        if let Some(errors) = response.errors {
            if let Some(error) = errors.first() {
                return Err(error.message.clone().into());
            }
        }

        if let Some(data) = response.data {
            if let Some(submit_transaction) = data.submit_transaction {
                return Ok(submit_transaction.hash);
            }
        }

        Err("Failed to broadcast transaction - no data or hash returned".into())
    }
}

impl<C: Client> CardanoClient<C> {
    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.get_tip_number().await
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<primitives::Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

impl<C: Client> ChainStaking for CardanoClient<C> {}

impl<C: Client> ChainPerpetual for CardanoClient<C> {}


impl<C: Client> ChainTraits for CardanoClient<C> {}
