use std::error::Error;

use primitives::{Asset, Chain};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Account, Block, Embedded, NodeStatus, Payment};

pub struct StellarClient {
    url: String,
    client: ClientWithMiddleware,
}

impl StellarClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Stellar
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_node_status().await?.history_latest_ledger)
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }

    pub async fn get_node_status(&self) -> Result<NodeStatus, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/", self.url);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/ledgers/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account(&self, account_id: String) -> Result<Account, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/accounts/{}", self.url, account_id);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_account_payments(&self, account_id: String) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let query = [
            ("order", "desc".to_string()),
            ("limit", 200.to_string()),
            ("include_failed", "true".to_string()),
        ];
        let url = format!("{}/accounts/{}/payments", self.url, account_id);
        Ok(self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .json::<Embedded<Payment>>()
            .await?
            ._embedded
            .records)
    }

    pub async fn get_block_payments(&self, block_number: i64, limit: usize, cursor: Option<String>) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let query = [
            ("limit", limit.to_string()),
            ("include_failed", "true".to_string()),
            ("cursor", cursor.unwrap_or_default()),
        ];
        let url = format!("{}/ledgers/{}/payments", self.url, block_number);
        Ok(self
            .client
            .get(url)
            .query(&query)
            .send()
            .await?
            .json::<Embedded<Payment>>()
            .await?
            ._embedded
            .records)
    }

    pub async fn get_block_payments_all(&self, block_number: i64) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let mut results: Vec<Payment> = Vec::new();
        let mut cursor: Option<String> = None;
        let limit: usize = 200;
        loop {
            let payments = self.get_block_payments(block_number, limit, cursor).await?;
            results.extend(payments.clone());
            cursor = payments.last().map(|x| x.id.clone());

            if payments.len() < limit {
                return Ok(results);
            }
        }
    }
}
