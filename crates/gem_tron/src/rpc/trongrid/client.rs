use crate::rpc::model::{Transaction, TransactionReceiptData};
use crate::rpc::trongrid::model::{Data, TronGridAccount};
use crate::rpc::TronClient;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use std::error::Error;
use std::result::Result;

#[derive(Debug, Clone)]
pub struct TronGridClient {
    tron_client: TronClient,
    client: Client,
    url: String,
}

impl TronGridClient {
    pub fn new(tron_client: TronClient, url: String, key: String) -> Self {
        Self {
            tron_client,
            client: Client::builder()
                .default_headers({
                    let mut headers = HeaderMap::new();
                    if let Ok(key_value) = HeaderValue::from_str(&key) {
                        headers.insert("TRON-PRO-API-KEY", key_value);
                    }
                    headers
                })
                .build()
                .unwrap(),
            url,
        }
    }

    pub async fn get_transactions_by_address(&self, address: &str) -> Result<Data<Vec<Transaction>>, Box<dyn Error + Send + Sync>> {
        let url = &format!("{}/v1/accounts/{}/transactions", self.url, address);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_accounts_by_address(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        let url = &format!("{}/v1/accounts/{}", self.url, address);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_transactions_reciepts(&self, transaction_ids: Vec<String>) -> Result<Vec<TransactionReceiptData>, Box<dyn Error + Send + Sync>> {
        let mut reciepts = Vec::new();
        for transaction_id in transaction_ids {
            let reciept = self.tron_client.get_transaction_reciept(transaction_id.clone()).await?;
            reciepts.push(reciept);
        }
        Ok(reciepts)
    }
}
