use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use jsonrpsee::{
    core::{client::ClientT, params::ObjectParams},
    http_client::{HttpClient, HttpClientBuilder},
};
use primitives::{chain::Chain, Transaction};

use super::model::Block;

pub struct NearClient {
    client: HttpClient,
}

impl NearClient {
    pub fn new(url: String) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(256 * 1024 * 1024) // 256MB
            .build(url)
            .unwrap();

        Self { client }
    }
}

#[async_trait]
impl ChainProvider for NearClient {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let mut params = ObjectParams::new();
        params.insert("finality", "final").unwrap();

        let block: Block = self.client.request("block", params).await?;
        Ok(block.header.height)
    }

    async fn get_transactions(
        &self,
        _block_number: i64,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        //TODO: Implement fetching transactions
        Ok(vec![])
    }
}
