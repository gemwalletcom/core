use std::{error::Error, str::FromStr};

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use ethers::providers::{JsonRpcClient, Http, RetryClientBuilder, RetryClient};
use num_traits::Num;
use primitives::{chain::Chain, TransactionType, TransactionState, TransactionDirection, AssetId};
use reqwest::Url;
use serde_json::json;
use super::model::{Block, Transaction, TransactionReciept};
use num_bigint::BigUint;

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
        let to: ethers::types::Address = transaction.clone().to.unwrap_or_default().clone().parse().unwrap_or_default();
        let from = ethers::utils::to_checksum(&from, None);
        
        // system transfer
        if transaction.input == "0x" {
            let transaction = primitives::Transaction::new( 
                transaction.hash.clone(),
                self.chain.as_asset_id(), 
                from, 
                ethers::utils::to_checksum(&to, None),
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
                Utc::now()
            );
            return Some(transaction);
        }
        // ERC20 transfer. Only add confirmed
        if transaction.input.starts_with("0xa9059cbb") && state == TransactionState::Confirmed {
            let token_id = ethers::utils::to_checksum(&to, None);
            let asset_id = AssetId{chain: self.chain, token_id: Some(token_id)};
            let value: String = transaction.input.chars().skip(74).take(64).collect();
            let to_address: ethers::types::Address = transaction.input.chars().skip(34).take(40).collect::<String>().parse().unwrap();
            let to_address = ethers::utils::to_checksum(&to_address, None);
            let value = BigUint::from_str_radix(value.as_str(), 16).unwrap_or_default();

            let transaction = primitives::Transaction::new( 
                transaction.hash.clone(),
                asset_id, 
                from, 
                to_address.to_string(),
                None,
                TransactionType::Transfer, 
                state, 
                block.to_string(),
                nonce.to_string(), 
                fee.to_string(),
                self.chain.as_asset_id(), 
                value.to_string(),
                None,
                TransactionDirection::SelfTransfer, 
                Utc::now()
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
        let transactions = block.transactions.into_iter().filter(|x| 
            x.input == "0x" || x.input.starts_with("0xa9059cbb")
        ).collect::<Vec<Transaction>>();
        let hashes = transactions.clone().into_iter().map(|x| x.hash).collect();
        let reciepts = self.get_transaction_reciepts(hashes).await?;

        let transactions = transactions.into_iter().zip(reciepts.iter()).filter_map(|(transaction, receipt)| {
            self.map_transaction(transaction, receipt)
        }).collect::<Vec<primitives::Transaction>>();

        return Ok(transactions);
    }
}