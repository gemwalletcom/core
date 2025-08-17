use super::model::Block;
use crate::rpc::model::{Account, AssetResponse, BlockHeaders, Transactions, TransactionsParams};
use gem_client::ContentType;
use std::collections::HashMap;
use std::error::Error;

#[cfg(feature = "rpc")]
use chain_traits::{ChainPerpetual, ChainStaking, ChainTraits};
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use primitives::Chain;

#[derive(Debug)]
pub struct AlgorandClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> AlgorandClient<C> {
    pub fn new(client: C) -> Self {
        Self {
            client,
            chain: Chain::Algorand,
        }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_block_headers(&self) -> Result<BlockHeaders, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v2/block-headers").await?)
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/accounts/{}", address)).await?)
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<AssetResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/assets/{}", asset_id)).await?)
    }

    pub async fn get_account_transactions(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/accounts/{}/transactions", address)).await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/blocks/{}", block_number)).await?)
    }

    pub async fn get_transactions_params(&self) -> Result<TransactionsParams, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v2/transactions/params").await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<super::model::TransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let headers = Some(HashMap::from([(
            "Content-Type".to_string(),
            ContentType::ApplicationXBinary.as_str().to_string(),
        )]));

        Ok(self.client.post("/v2/transactions", &data, headers).await?)
    }

    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<super::model::TransactionStatus, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/transactions/pending/{}", transaction_id)).await?)
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> ChainStaking for AlgorandClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainPerpetual for AlgorandClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> chain_traits::ChainAccount for AlgorandClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainTraits for AlgorandClient<C> {}
