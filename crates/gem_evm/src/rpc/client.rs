use alloy_primitives::{hex, Address, Bytes};
use anyhow::anyhow;
#[cfg(feature = "reqwest")]
use gem_jsonrpc::{
    types::{JsonRpcError, JsonRpcResult},
    JsonRpcClient,
};
#[cfg(not(feature = "reqwest"))]
use gem_jsonrpc::{
    types::{JsonRpcError, JsonRpcResult},
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::any::TypeId;
use std::str::FromStr;

use crate::rpc::model::{BlockTransactionsIds, TransactionReplayTrace};

use super::model::{Block, Transaction, TransactionReciept};
use primitives::{Chain, EVMChain};

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

#[cfg(feature = "reqwest")]
#[derive(Clone)]
pub struct EthereumClient {
    pub chain: EVMChain,
    client: JsonRpcClient,
}

#[cfg(feature = "reqwest")]
impl EthereumClient {
    pub fn new(chain: EVMChain, url: &str) -> Self {
        let client = JsonRpcClient::new_reqwest(url.to_string());
        Self { chain, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain.to_chain()
    }

    pub async fn eth_call<T: DeserializeOwned + 'static>(&self, contract_address: &str, call_data: &str) -> Result<T, anyhow::Error> {
        let to_address = Address::from_str(contract_address)?;

        let params = json!([
            {
                "to": to_address.to_string(),
                "data": call_data
            },
            "latest"
        ]);

        let result: String = self.client.call("eth_call", params).await?;
        let result_bytes = Bytes::from(hex::decode(&result).map_err(|e| anyhow!(e))?);

        // Deserialize T (hex string or struct) from the returned bytes.
        if TypeId::of::<T>() == TypeId::of::<String>() {
            Ok(serde_json::from_value(serde_json::Value::String(result_bytes.to_string()))?)
        } else {
            serde_json::from_slice(&result_bytes).map_err(|e| anyhow!(e))
        }
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, JsonRpcError> {
        let params = json!([format!("0x{:x}", block_number), true]);
        self.client.call("eth_getBlockByNumber", params).await
    }

    pub async fn get_block_receipts(&self, block_number: i64) -> Result<Vec<TransactionReciept>, JsonRpcError> {
        let params = json!([format!("0x{:x}", block_number)]);
        self.client.call("eth_getBlockReceipts", params).await
    }

    pub async fn get_latest_block(&self) -> Result<i64, anyhow::Error> {
        let block_hex: String = self.client.call("eth_blockNumber", json!([])).await?;
        let block_hex = block_hex.trim_start_matches("0x");
        i64::from_str_radix(block_hex, 16).map_err(|e| anyhow!("Invalid block number format: {}", e))
    }

    pub async fn get_blocks(&self, blocks: &[String], include_transactions: bool) -> Result<Vec<BlockTransactionsIds>, JsonRpcError> {
        let calls: Vec<(String, serde_json::Value)> = blocks
            .iter()
            .map(|block| ("eth_getBlockByNumber".to_string(), json!([block, include_transactions])))
            .collect();

        let results = self.client.batch_call::<BlockTransactionsIds>(calls).await?;
        let mut blocks_result = Vec::new();
        for result in results {
            match result {
                JsonRpcResult::Value(value) => blocks_result.push(value.result),
                JsonRpcResult::Error(error) => return Err(error.error),
            }
        }
        Ok(blocks_result)
    }

    pub async fn get_transactions(
        &self,
        hashes: &[String],
    ) -> Result<Vec<(BlockTransactionsIds, Transaction, TransactionReciept, TransactionReplayTrace)>, JsonRpcError> {
        let transactions = self.get_transactions_by_hash(hashes).await?;
        let reciepts = self.get_transactions_receipts(hashes).await?;
        let traces = self.trace_replay_transactions(hashes).await?;
        let block_ids = reciepts.iter().map(|x| x.block_number.clone()).collect::<Vec<String>>();
        let blocks = self.get_blocks(&block_ids, false).await?;

        Ok(blocks
            .into_iter()
            .zip(transactions.into_iter())
            .zip(reciepts.into_iter())
            .zip(traces.into_iter())
            .map(|(((block, tx), receipt), trace)| (block, tx, receipt, trace))
            .collect())
    }

    pub async fn get_transactions_by_hash(&self, hashes: &[String]) -> Result<Vec<Transaction>, JsonRpcError> {
        let calls: Vec<(String, serde_json::Value)> = hashes.iter().map(|hash| ("eth_getTransactionByHash".to_string(), json!([hash]))).collect();

        let results = self.client.batch_call::<Transaction>(calls).await?;
        let mut transactions = Vec::new();
        for result in results {
            match result {
                JsonRpcResult::Value(value) => transactions.push(value.result),
                JsonRpcResult::Error(error) => return Err(error.error),
            }
        }
        Ok(transactions)
    }

    pub async fn get_transactions_receipts(&self, hashes: &[String]) -> Result<Vec<TransactionReciept>, JsonRpcError> {
        let calls: Vec<(String, serde_json::Value)> = hashes.iter().map(|hash| ("eth_getTransactionReceipt".to_string(), json!([hash]))).collect();

        let results = self.client.batch_call::<TransactionReciept>(calls).await?;
        let mut receipts = Vec::new();
        for result in results {
            match result {
                JsonRpcResult::Value(value) => receipts.push(value.result),
                JsonRpcResult::Error(error) => return Err(error.error),
            }
        }
        Ok(receipts)
    }

    pub async fn trace_replay_block_transactions(&self, block_number: i64) -> Result<Vec<TransactionReplayTrace>, JsonRpcError> {
        let params = json!([format!("0x{:x}", block_number), json!(["stateDiff"])]);
        self.client.call("trace_replayBlockTransactions", params).await
    }

    pub async fn trace_replay_transactions(&self, tx_hash: &[String]) -> Result<Vec<TransactionReplayTrace>, JsonRpcError> {
        let calls: Vec<(String, serde_json::Value)> = tx_hash
            .iter()
            .map(|hash| ("trace_replayTransaction".to_string(), json!([hash, json!("stateDiff")])))
            .collect();
        let results = self.client.batch_call::<TransactionReplayTrace>(calls).await?;
        results.into_iter().try_fold(Vec::new(), |mut acc, result| {
            match result {
                JsonRpcResult::Value(value) => acc.push(value.result),
                JsonRpcResult::Error(error) => return Err(error.error),
            }
            Ok(acc)
        })
    }
}
