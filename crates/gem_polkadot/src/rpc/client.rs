use std::error::Error;

use primitives::{Asset, Chain};
use gem_client::Client;
use chain_traits::{ChainAccount, ChainPerpetual, ChainStaking, ChainTraits};

use super::model::{Block, BlockHeader};
use crate::models::account::PolkadotAccountBalance;
use crate::models::block::PolkadotNodeVersion;
use crate::models::transaction::PolkadotTransactionBroadcastResponse;

pub struct PolkadotClient<C: Client> {
    pub client: C,
}

impl<C: Client> PolkadotClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_balance(&self, address: String) -> Result<PolkadotAccountBalance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/accounts/{}/balance-info", address)).await?)
    }

    pub async fn get_node_version(&self) -> Result<PolkadotNodeVersion, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/node/version").await?)
    }

    pub async fn get_block_head(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/blocks/head").await?)
    }

    pub async fn get_blocks(&self, from: &str, to: &str) -> Result<Vec<Block>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/blocks?range={}-{}&noFees=true", from, to)).await?)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<PolkadotTransactionBroadcastResponse, Box<dyn Error + Send + Sync>> {
        let payload = serde_json::json!({ "tx": data });
        Ok(self.client.post("/transaction", &payload, None).await?)
    }

    pub async fn get_block_header(&self, block: &str) -> Result<BlockHeader, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/blocks/{}/header", block)).await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/blocks/{}", block_number)).await?)
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Polkadot
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}

impl<C: Client> ChainTraits for PolkadotClient<C> {}
impl<C: Client> ChainAccount for PolkadotClient<C> {}
impl<C: Client> ChainPerpetual for PolkadotClient<C> {}
impl<C: Client> ChainStaking for PolkadotClient<C> {}
