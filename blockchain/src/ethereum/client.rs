use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use ethers::providers::{JsonRpcClient, Http, RetryClientBuilder, RetryClient};
use primitives::{chain::Chain, TransactionType, TransactionState, TransactionDirection};
use reqwest::Url;
use serde_json::json;
use super::model::{Block, Transaction, TransactionReciept};

pub struct EthereumClient {
    chain: Chain,
    client: RetryClient<Http>,
}

impl EthereumClient {
    pub fn new(chain: Chain, url: String) -> Self {
        let provider = Http::new(Url::parse(url.as_str()).unwrap());
        let client = RetryClientBuilder::default()
            .build(provider, Box::<ethers::providers::HttpRateLimitRetryPolicy>::default());
        
        Self {
            chain,
            client,
        }
    }

    async fn get_transaction_reciept(&self, hash: &str) -> Result<TransactionReciept, Box<dyn Error + Send + Sync>> {
        let reciept: TransactionReciept = JsonRpcClient::request(&self.client, "eth_getTransactionReceipt", vec![json!(hash)]).await?;
        Ok(reciept)
    }

    async fn get_transaction_reciepts(&self, hashes: Vec<String>) -> Result<Vec<TransactionReciept>, Box<dyn Error + Send + Sync>> {
        let recieipts = futures::future::try_join_all(
            hashes.iter().map(|hash| { self.get_transaction_reciept(hash) 
        })).await?;
        Ok(recieipts)
    }

    async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let params = vec![json!(format!("0x{:x}", block_number)), json!(true)];
        let block: Block = JsonRpcClient::request(&self.client, "eth_getBlockByNumber", params).await?;         
        Ok(block)
    }

    fn map_transaction(&self, transaction: Transaction, reciept: &TransactionReciept) -> Option<primitives::Transaction> {
        let state = if reciept.status == "0x1" { TransactionState::Confirmed } else { TransactionState::Failed };
        let value = transaction.value.to_string();
        let nonce = transaction.nonce.as_i32();
        let block = transaction.block_number.as_i32();
        let fee = reciept.gas_used.clone().value * reciept.effective_gas_price.clone().value;
        let from: ethers::types::Address = transaction.from.clone().parse().unwrap();
        let to: ethers::types::Address = transaction.to.unwrap_or_default().clone().parse().unwrap_or_default();
        let from = ethers::utils::to_checksum(&from, None);
        let to = ethers::utils::to_checksum(&to, None);

        // system transfer
        if transaction.input == "0x" {
            let transaction = primitives::Transaction::new( 
                transaction.hash.clone(),
                self.chain.as_asset_id(), 
                from, 
                to,
                None,
                TransactionType::Transfer, 
                state, 
                block.to_string(),
                nonce.to_string(), 
                fee.to_string(),
                self.chain.as_asset_id(), 
                value,
                None,
                TransactionDirection::SelfTransfer, 
                Utc::now().naive_utc()
            );
            return Some(transaction);
        }
        None
    }
}

#[async_trait]
impl ChainProvider for EthereumClient {

    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = JsonRpcClient::request(&self.client, "eth_blockNumber", ()).await?;        
        let block_number = i64::from_str_radix(&block[2..], 16)?;
        Ok(block_number)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        // filter out non transfer transactions
        let transactions = block.transactions.into_iter().filter(|x| x.input == "0x").collect::<Vec<Transaction>>();
        let hashes = transactions.clone().into_iter().map(|x| x.hash).collect();
        let reciepts = self.get_transaction_reciepts(hashes).await?;

        let transactions = transactions.into_iter().zip(reciepts.iter()).filter_map(|(transaction, receipt)| {
            self.map_transaction(transaction, receipt)
        }).collect::<Vec<primitives::Transaction>>();

        return Ok(transactions);
    }
}