use std::error::Error;

use crate::models::account::{Account, StellarAccount};
use crate::models::common::{Embedded, StellarAsset, StellarEmbedded};
use crate::models::fee::StellarFees;
use crate::models::node::NodeStatus;
use crate::models::transaction::{Payment, StellarTransactionBroadcast, StellarTransactionStatus};

use chain_traits::{ChainAddressStatus, ChainPerpetual, ChainProvider, ChainStaking, ChainTraits};
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
        Self { client, chain: Chain::Stellar }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_node_status(&self) -> Result<NodeStatus, Box<dyn Error + Send + Sync>> {
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
        let headers = Some(HashMap::from([(
            "Content-Type".to_string(),
            ContentType::ApplicationFormUrlEncoded.as_str().to_string(),
        )]));

        Ok(self.client.post("/transactions", &body, headers).await?)
    }

    pub async fn get_assets_by_issuer(&self, issuer: &str) -> Result<StellarEmbedded<StellarAsset>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/assets?asset_issuer={}&limit=200", issuer)).await?)
    }

    pub async fn get_account(&self, account_id: String) -> Result<Account, Box<dyn Error + Send + Sync>> {
        let url = format!("/accounts/{}", account_id);
        Ok(self.client.get(&url).await?)
    }

    pub async fn get_account_payments(&self, account_id: String) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let url = format!("/accounts/{}/payments?order=desc&limit=200&include_failed=true", account_id);
        let result: Embedded<Payment> = self.client.get(&url).await?;
        Ok(result._embedded.records)
    }

    pub async fn get_block_payments(&self, block_number: i64, limit: usize, cursor: Option<String>) -> Result<Vec<Payment>, Box<dyn Error + Send + Sync>> {
        let cursor_param = cursor.unwrap_or_default();
        let url = format!("/ledgers/{}/payments?limit={}&include_failed=true&cursor={}", block_number, limit, cursor_param);
        let result: Embedded<Payment> = self.client.get(&url).await?;
        Ok(result._embedded.records)
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

impl<C: Client> ChainStaking for StellarClient<C> {}

impl<C: Client> ChainPerpetual for StellarClient<C> {}

impl<C: Client> ChainAddressStatus for StellarClient<C> {}

impl<C: Client> chain_traits::ChainAccount for StellarClient<C> {}

impl<C: Client> ChainTraits for StellarClient<C> {}

impl<C: Client> ChainProvider for StellarClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        self.chain
    }
}
