use std::error::Error;

use crate::typeshare::account::StellarAccount;
use crate::typeshare::node::StellarNodeStatus;
use crate::typeshare::fee::StellarFees;
use crate::typeshare::transaction::{StellarTransactionBroadcast, StellarTransactionStatus};
use crate::typeshare::common::{StellarAsset, StellarEmbedded};

use chain_traits::{ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::{Client, ContentType};
use primitives::Chain;
use std::collections::HashMap;

#[derive(Debug)]
pub struct StellarClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> StellarClient<C> {
    pub fn new(client: C) -> Self {
        Self { 
            client, 
            chain: Chain::Stellar 
        }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_node_status(&self) -> Result<StellarNodeStatus, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/").await?)
    }


    pub async fn get_stellar_account(&self, address: &str) -> Result<StellarAccount, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/accounts/{}", address)).await?)
    }




    pub async fn get_transaction_status(&self, transaction_id: &str) -> Result<StellarTransactionStatus, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/transactions/{}", transaction_id)).await?)
    }

    pub async fn get_fees(&self) -> Result<StellarFees, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/fee_stats").await?)
    }

    pub async fn broadcast_transaction(&self, data: &str) -> Result<StellarTransactionBroadcast, Box<dyn Error + Send + Sync>> {
        let encoded_data = urlencoding::encode(data);
        let body = format!("tx={}", encoded_data);
        let headers = Some(HashMap::from([
            ("Content-Type".to_string(), ContentType::ApplicationFormUrlEncoded.as_str().to_string())
        ]));
        
        Ok(self.client.post("/transactions", &body, headers).await?)
    }

    pub async fn get_assets_by_issuer(&self, issuer: &str) -> Result<StellarEmbedded<StellarAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self.client
            .get(&format!("/assets?asset_issuer={}&limit=200", issuer))
            .await?)
    }

    pub async fn get_account(&self, account_id: String) -> Result<super::model::Account, Box<dyn Error + Send + Sync>> {
        let url = format!("/accounts/{}", account_id);
        Ok(self.client.get(&url).await?)
    }

    pub async fn get_account_payments(&self, account_id: String) -> Result<Vec<super::model::Payment>, Box<dyn Error + Send + Sync>> {
        let url = format!("/accounts/{}/payments?order=desc&limit=200&include_failed=true", account_id);
        let result: super::model::Embedded<super::model::Payment> = self.client.get(&url).await?;
        Ok(result._embedded.records)
    }

    pub async fn get_block_payments(&self, block_number: i64, limit: usize, cursor: Option<String>) -> Result<Vec<super::model::Payment>, Box<dyn Error + Send + Sync>> {
        let cursor_param = cursor.unwrap_or_default();
        let url = format!("/ledgers/{}/payments?limit={}&include_failed=true&cursor={}", block_number, limit, cursor_param);
        let result: super::model::Embedded<super::model::Payment> = self.client.get(&url).await?;
        Ok(result._embedded.records)
    }

    pub async fn get_block_payments_all(&self, block_number: i64) -> Result<Vec<super::model::Payment>, Box<dyn Error + Send + Sync>> {
        let mut results: Vec<super::model::Payment> = Vec::new();
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

impl<C: Client> ChainStaking for StellarClient<C> {}

impl<C: Client> ChainPerpetual for StellarClient<C> {}

impl<C: Client> chain_traits::ChainAccount for StellarClient<C> {}

impl<C: Client> ChainTraits for StellarClient<C> {}
