use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use jsonrpsee::{http_client::{HttpClientBuilder, HttpClient}, core::{client::ClientT, params::BatchRequestBuilder}, rpc_params};
use num_traits::Num;
use primitives::{chain::Chain, TransactionType, TransactionState, AssetId};
use serde_json::json;
use super::model::{Block, Transaction, TransactionReciept};
use num_bigint::BigUint;

const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
const FUNCTION_ERC20_APPROVE: &str = "0x095ea7b3";

pub struct EthereumClient {
    chain: Chain,
    client: HttpClient,
}

impl EthereumClient {
    pub fn new(chain: Chain, url: String) -> Self {
        let client = HttpClientBuilder::default().build(&url).unwrap();
        
        Self {
            chain,
            client,
        }
    }

    async fn get_transaction_reciepts(&self, hashes: Vec<String>) -> Result<Vec<TransactionReciept>, Box<dyn Error + Send + Sync>> {
        let mut batch = BatchRequestBuilder::new();
	    for hash in hashes.iter() {
            batch.insert("eth_getTransactionReceipt", vec![json!(hash)]).unwrap();
        }
        let response = self.client.batch_request::<TransactionReciept>(batch).await?;
        let reciepts = response.iter().filter_map(|r| r.as_ref().ok()).cloned().collect::<Vec<TransactionReciept>>();
        if reciepts.len() != hashes.len() {
            return Err("Failed to get all transaction reciepts".into());
        }
        Ok(reciepts)
    }

    async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let params = vec![json!(format!("0x{:x}", block_number)), json!(true)];
        Ok(self.client.request("eth_getBlockByNumber", params).await?)
    }

    fn map_transaction(&self, transaction: Transaction, reciept: &TransactionReciept) -> Option<primitives::Transaction> {
        let state = if reciept.status == "0x1" { TransactionState::Confirmed } else { TransactionState::Failed };
        let value = transaction.value.to_string();
        let nonce = transaction.nonce.as_i32();
        let block = transaction.block_number.as_i32();
        let fee = reciept.gas_used.clone().value * reciept.effective_gas_price.clone().value;
        let from: alloy_primitives::Address = transaction.from.clone().parse().unwrap();
        let to: alloy_primitives::Address = transaction.clone().to.unwrap_or_default().clone().parse().unwrap_or_default();
        let from = alloy_primitives::Address::to_checksum(&from, None);
        
        // system transfer
        if transaction.input == "0x" {
            let transaction = primitives::Transaction::new( 
                transaction.hash.clone(),
                self.chain.as_asset_id(), 
                from, 
                alloy_primitives::Address::to_checksum(&to, None),
                None,
                TransactionType::Transfer, 
                state, 
                block.to_string(),
                nonce.to_string(), 
                fee.to_string(),
                self.chain.as_asset_id(), 
                value,
                None,
                Utc::now()
            );
            return Some(transaction);
        }
        // ERC20 transfer. Only add confirmed
        let input_prefix = transaction.input.chars().take(10).collect::<String>();
        if (input_prefix.starts_with(FUNCTION_ERC20_TRANSFER) || input_prefix.starts_with(FUNCTION_ERC20_APPROVE)) && state == TransactionState::Confirmed {
            let transaction_type = match input_prefix.as_str() {
                FUNCTION_ERC20_TRANSFER => TransactionType::Transfer,
                FUNCTION_ERC20_APPROVE => TransactionType::TokenApproval,
                _ => TransactionType::Transfer,
            };
            let token_id = alloy_primitives::Address::to_checksum(&to, None);
            let asset_id = AssetId{chain: self.chain, token_id: Some(token_id)};
            let value: String = transaction.input.chars().skip(74).take(64).collect();
            let to_address: alloy_primitives::Address = transaction.input.chars().skip(34).take(40).collect::<String>().parse().unwrap();
            let to_address = alloy_primitives::Address::to_checksum(&to_address, None);
            let value = BigUint::from_str_radix(value.as_str(), 16).unwrap_or_default();

            let transaction = primitives::Transaction::new( 
                transaction.hash.clone(),
                asset_id, 
                from, 
                to_address.to_string(),
                None,
                transaction_type, 
                state, 
                block.to_string(),
                nonce.to_string(), 
                fee.to_string(),
                self.chain.as_asset_id(), 
                value.to_string(),
                None,
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
        let block: String = self.client.request( "eth_blockNumber", rpc_params![]).await?;        
        let block_number = i64::from_str_radix(&block[2..], 16)?;
        Ok(block_number)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions = block.transactions;
        let hashes = transactions.clone().into_iter().map(|x| x.hash).collect();
        let reciepts = self.get_transaction_reciepts(hashes).await?;

        let transactions = transactions.into_iter().zip(reciepts.iter()).filter_map(|(transaction, receipt)| {
            self.map_transaction(transaction, receipt)
        }).collect::<Vec<primitives::Transaction>>();

        return Ok(transactions);
    }
}