use crate::rpc::model::{Transaction, TransactionReceiptData};
use crate::rpc::trongrid::model::{Data, Trc20Transaction, TronGridAccount};
use crate::rpc::client::TronClient;
use gem_client::Client;
use std::error::Error;
use std::result::Result;

#[derive(Clone)]
pub struct TronGridClient<C: Client> {
    tron_client: TronClient<C>,
    client: C,
}

impl<C: Client> TronGridClient<C> {
    pub fn new(tron_client: TronClient<C>, client: C, _url: String, _key: String) -> Self {
        Self {
            tron_client,
            client,
        }
    }

    pub async fn get_transactions_by_address(&self, address: &str, limit: i32) -> Result<Data<Vec<Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions?limit={}", address, limit);
        let result = self.client.get(path).await?;
        Ok(result)
    }

    pub async fn get_transactions_by_address_trc20(&self, address: &str, limit: i32) -> Result<Data<Vec<Trc20Transaction>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}/transactions/trc20?limit={}", address, limit);
        let result = self.client.get(path).await?;
        Ok(result)
    }

    pub async fn get_accounts_by_address(&self, address: &str) -> Result<Data<Vec<TronGridAccount>>, Box<dyn Error + Send + Sync>> {
        let path = &format!("/v1/accounts/{}", address);
        let result = self.client.get(path).await?;
        Ok(result)
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
