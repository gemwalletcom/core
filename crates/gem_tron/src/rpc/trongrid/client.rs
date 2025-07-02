use crate::rpc::model::{Transaction, TransactionReceiptData};
use crate::rpc::trongrid::model::{Data, Trc20Transaction, TronGridAccount};
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

    pub async fn get_transactions_by_address(&self, address: &str, limit: i32) -> Result<Data<Vec<Transaction>>, Box<dyn Error + Send + Sync>> {
        let url = &format!("{}/v1/accounts/{}/transactions?limit={}", self.url, address, limit);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_transactions_by_address_trc20(&self, address: &str, limit: i32) -> Result<Data<Vec<Trc20Transaction>>, Box<dyn Error + Send + Sync>> {
        let url = &format!("{}/v1/accounts/{}/transactions/trc20?limit={}", self.url, address, limit);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_accounts_by_address(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        let url = &format!("{}/v1/accounts/{}", self.url, address);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_transactions(&self, transaction_ids: Vec<String>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let mut transactions = Vec::new();
        for transaction_id in transaction_ids {
            let transaction = self.tron_client.get_transaction(transaction_id.clone()).await?;
            transactions.push(transaction);
        }
        Ok(transactions)
    }

    pub async fn get_token_transactions(&self, address: &str, limit: i32) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let token_transfers_ids = self
            .get_transactions_by_address_trc20(address, limit)
            .await?
            .data
            .into_iter()
            .map(|x| x.transaction_id.clone())
            .collect::<Vec<String>>();
        self.get_transactions(token_transfers_ids).await
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
