use crate::models::Transaction;
use crate::rpc::trongrid::model::{Data, Trc20Transaction, TronGridAccount};
use gem_client::Client;
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;

#[derive(Clone)]
pub struct TronGridClient<C: Client> {
    client: C,
    api_key: String,
}

impl<C: Client> TronGridClient<C> {
    pub fn new(client: C, api_key: String) -> Self {
        Self { client, api_key }
    }

    fn headers(&self) -> Option<HashMap<String, String>> {
        if self.api_key.is_empty() {
            None
        } else {
            let mut headers = HashMap::new();
            headers.insert("TRON-PRO-API-KEY".to_string(), self.api_key.clone());
            Some(headers)
        }
    }

    pub async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Data<Vec<Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions?limit={}", address, limit);
        Ok(self.client.get_with_headers(path, self.headers()).await?)
    }

    pub async fn get_transactions_by_address_trc20(&self, address: &str, limit: usize) -> Result<Data<Vec<Trc20Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions/trc20?limit={}", address, limit);
        Ok(self.client.get_with_headers(path, self.headers()).await?)
    }

    pub async fn get_accounts_by_address(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}", address);
        Ok(self.client.get_with_headers(path, self.headers()).await?)
    }
}
