use alloy_primitives::{Address, Bytes as AlloyBytes};
use alloy_rpc_client::{ClientBuilder, RpcClient};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionRequest as AlloyTransactionRequest};
use anyhow::{anyhow, Result};
use futures::future::try_join_all;
use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::error::Error;
use std::str::FromStr;
use url::Url;

use crate::rpc::model::BlockTransactionsIds;

use super::model::{Block, Transaction, TransactionReciept};
use primitives::{Chain, EVMChain};

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

#[derive(Clone)]
pub struct EthereumClient {
    pub chain: EVMChain,
    client: RpcClient,
}

impl EthereumClient {
    pub fn new(chain: EVMChain, url_str: String) -> Self {
        let url: Url = Url::parse(&url_str).expect("Invalid Ethereum node URL");
        let client = ClientBuilder::default().http(url);
        Self { chain, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain.to_chain()
    }

    pub async fn eth_call<T: DeserializeOwned + 'static>(&self, contract_address: &str, call_data: &str) -> Result<T> {
        let to_address = Address::from_str(contract_address)?;
        let data_bytes = AlloyBytes::from_str(call_data)?;

        let tx_request = AlloyTransactionRequest {
            to: Some(alloy_primitives::TxKind::Call(to_address)),
            input: data_bytes.into(),
            ..Default::default()
        };

        let params = (tx_request, BlockId::Number(BlockNumberOrTag::Latest));
        let result_bytes: AlloyBytes = self.client.request("eth_call", params).await?;

        // Deserialize T (hex string or struct) from the returned bytes.
        if TypeId::of::<T>() == TypeId::of::<String>() {
            Ok(serde_json::from_value(serde_json::Value::String(result_bytes.to_string()))?)
        } else {
            serde_json::from_slice(&result_bytes).map_err(|e| anyhow!(e))
        }
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block> {
        let block_id = BlockId::Number(BlockNumberOrTag::Number(block_number as u64));
        let include_txs = true;
        let params = (block_id, include_txs);

        Ok(self.client.request("eth_getBlockByNumber", params).await?)
    }

    pub async fn get_block_receipts(&self, block_number: i64) -> Result<Vec<TransactionReciept>> {
        let block_id = BlockId::Number(BlockNumberOrTag::Number(block_number as u64));
        let params = (block_id,);

        Ok(self.client.request("eth_getBlockReceipts", params).await?)
    }

    pub async fn get_block_with_codes(&self, block_number: i64) -> Result<(Block, Vec<TransactionReciept>, Vec<bool>), Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions_reciepts = self.get_block_receipts(block_number).await?;

        let to_addresses: Vec<String> = block.transactions.iter().filter_map(|tx| tx.to.clone()).collect();
        let codes = self.get_codes(to_addresses).await?;
        let has_code_flags: Vec<bool> = codes.iter().map(|code| !code.is_empty() && code != "0x").collect();

        Ok((block, transactions_reciepts, has_code_flags))
    }

    pub async fn get_latest_block(&self) -> Result<i64> {
        let block_hex: String = self.client.request("eth_blockNumber", ()).await?;
        if !block_hex.starts_with("0x") {
            return Err(anyhow!("Invalid block number format: {}", block_hex));
        }
        Ok(i64::from_str_radix(&block_hex[2..], 16)?)
    }

    pub async fn get_blocks(&self, blocks: Vec<String>, include_transactions: bool) -> Result<Vec<BlockTransactionsIds>, Box<dyn Error + Send + Sync>> {
        let mut batch = self.client.new_batch();
        let mut futures = Vec::new();
        for block in &blocks {
            futures.push(batch.add_call("eth_getBlockByNumber", &(block, include_transactions))?);
        }
        batch.send().await?;
        Ok(try_join_all(futures).await?)
    }

    pub async fn get_transactions(
        &self,
        hashes: Vec<String>,
    ) -> Result<Vec<(BlockTransactionsIds, Transaction, TransactionReciept)>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_transactions_by_hash(hashes.clone()).await?;
        let reciepts = self.get_transactions_receipts(hashes.clone()).await?;
        let block_ids = transactions.iter().map(|x| x.block_number.clone()).collect::<Vec<String>>();
        let blocks = self.get_blocks(block_ids.clone(), false).await?;

        Ok(blocks
            .into_iter()
            .zip(transactions.into_iter())
            .zip(reciepts.into_iter())
            .map(|((block, tx), receipt)| (block, tx, receipt))
            .collect())
    }

    pub async fn get_transactions_by_hash(&self, hashes: Vec<String>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let mut batch = self.client.new_batch();
        let mut futures = Vec::new();
        for hash in &hashes {
            futures.push(batch.add_call("eth_getTransactionByHash", &(hash,))?);
        }
        batch.send().await?;
        Ok(try_join_all(futures).await?)
    }

    pub async fn get_transactions_receipts(&self, hashes: Vec<String>) -> Result<Vec<TransactionReciept>, Box<dyn Error + Send + Sync>> {
        let mut batch = self.client.new_batch();
        let mut futures = Vec::new();
        for hash in &hashes {
            futures.push(batch.add_call("eth_getTransactionReceipt", &(hash,))?);
        }
        batch.send().await?;
        Ok(try_join_all(futures).await?)
    }

    pub async fn eth_get_code(&self, address: &str) -> Result<String> {
        let to_address = Address::from_str(address)?;
        let params = (to_address, BlockId::Number(BlockNumberOrTag::Latest));
        let code: String = self.client.request("eth_getCode", params).await?;
        Ok(code)
    }

    pub async fn get_codes(&self, addresses: Vec<String>) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut batch = self.client.new_batch();
        let mut futures = Vec::new();
        for address in &addresses {
            if let Ok(to_address) = Address::from_str(address) {
                let params = (to_address, BlockId::Number(BlockNumberOrTag::Latest));
                futures.push(batch.add_call("eth_getCode", &params)?);
            } else {
                futures.push(batch.add_call(
                    "eth_getCode",
                    &("0x0000000000000000000000000000000000000000", BlockId::Number(BlockNumberOrTag::Latest)),
                )?);
            }
        }
        batch.send().await?;
        Ok(try_join_all(futures).await?)
    }

    pub async fn get_transactions_with_codes(
        &self,
        hashes: Vec<String>,
    ) -> Result<Vec<(BlockTransactionsIds, Transaction, TransactionReciept, bool)>, Box<dyn Error + Send + Sync>> {
        let transactions = self.get_transactions_by_hash(hashes.clone()).await?;
        let reciepts = self.get_transactions_receipts(hashes.clone()).await?;
        let block_ids = transactions.iter().map(|x| x.block_number.clone()).collect::<Vec<String>>();
        let blocks = self.get_blocks(block_ids.clone(), false).await?;

        let to_addresses: Vec<String> = transactions.iter().filter_map(|tx| tx.to.clone()).collect();
        let codes = self.get_codes(to_addresses).await?;
        let has_code_flags: Vec<bool> = codes.iter().map(|code| !code.is_empty() && code != "0x").collect();

        Ok(blocks
            .into_iter()
            .zip(transactions.into_iter())
            .zip(reciepts.into_iter())
            .zip(has_code_flags.into_iter())
            .map(|(((block, tx), receipt), has_code)| (block, tx, receipt, has_code))
            .collect())
    }
}
