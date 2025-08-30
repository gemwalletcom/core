use crate::models::Transaction;
use crate::rpc::trongrid::model::{Data, Trc20Transaction, TronGridAccount};
use gem_client::Client;
use std::error::Error;
use std::result::Result;

#[derive(Clone)]
pub struct TronGridClient<C: Client> {
    client: C,
}

impl<C: Client> TronGridClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Data<Vec<Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions?limit={}", address, limit);
        Ok(self.client.get(path).await?)
    }

    pub async fn get_transactions_by_address_trc20(&self, address: &str, limit: usize) -> Result<Data<Vec<Trc20Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions/trc20?limit={}", address, limit);
        Ok(self.client.get(path).await?)
    }

    pub async fn get_accounts_by_address(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}", address);
        Ok(self.client.get(path).await?)
    }

    pub async fn get_token_transactions(&self, address: &str, limit: usize) -> Result<Vec<Trc20Transaction>, Box<dyn Error + Send + Sync>> {
        let token_transfers = self.get_transactions_by_address_trc20(address, limit).await?;
        Ok(token_transfers.data)
    }
}
