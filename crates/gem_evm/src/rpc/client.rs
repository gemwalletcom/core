use alloy_primitives::{hex, Address, Bytes};
use anyhow::anyhow;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
use gem_jsonrpc::types::{JsonRpcError, JsonRpcResult};

use serde::de::DeserializeOwned;
use serde_json::json;
use std::any::TypeId;
use std::str::FromStr;

use crate::models::fee::EthereumFeeHistory;
use crate::rpc::model::{BlockTransactionsIds, TransactionReplayTrace};

use super::model::{Block, Transaction, TransactionReciept};
use primitives::{Chain, EVMChain};

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

#[derive(Clone)]
pub struct EthereumClient<C: Client + Clone> {
    pub chain: EVMChain,
    pub client: GenericJsonRpcClient<C>,
}

impl<C: Client + Clone> EthereumClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>, chain: EVMChain) -> Self {
        Self { chain, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain.to_chain()
    }

    pub async fn call<T: DeserializeOwned + 'static>(&self, method: String, params: serde_json::Value) -> Result<T, JsonRpcError> {
        self.client.call(&method, params).await
    }

    pub async fn batch_call<T: DeserializeOwned + 'static>(&self, calls: Vec<(String, serde_json::Value)>) -> Result<Vec<JsonRpcResult<T>>, JsonRpcError> {
        Ok(self.client.batch_call::<T>(calls).await?.into_iter().collect())
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
        Ok(self.client.batch_call::<TransactionReciept>(calls).await?.extract())
    }

    pub async fn get_transaction_receipt(&self, hash: &str) -> Result<TransactionReciept, JsonRpcError> {
        let params = json!([hash]);
        self.client.call("eth_getTransactionReceipt", params).await
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
        Ok(self.client.batch_call::<TransactionReplayTrace>(calls).await?.extract())
    }

    pub async fn get_eth_balance(&self, address: &str) -> Result<String, anyhow::Error> {
        let params = json!([address, "latest"]);
        Ok(self.client.call("eth_getBalance", params).await?)
    }

    pub async fn get_chain_id(&self) -> Result<String, anyhow::Error> {
        Ok(self.client.call("eth_chainId", json!([])).await?)
    }

    pub async fn get_block_number(&self) -> Result<String, anyhow::Error> {
        Ok(self.client.call("eth_blockNumber", json!([])).await?)
    }

    pub async fn get_transaction_count(&self, address: &str) -> Result<String, anyhow::Error> {
        let params = json!([address, "latest"]);
        Ok(self.client.call("eth_getTransactionCount", params).await?)
    }

    pub async fn send_raw_transaction(&self, data: &str) -> Result<String, JsonRpcError> {
        let params = json!([data]);
        self.client.call("eth_sendRawTransaction", params).await
    }

    pub async fn batch_eth_call<const N: usize>(
        &self,
        contract_address: &str,
        function_selectors: [&str; N],
    ) -> Result<[String; N], Box<dyn std::error::Error + Sync + Send>> {
        let calls: Vec<(String, serde_json::Value)> = function_selectors
            .iter()
            .map(|selector| ("eth_call".to_string(), json!([{"to": contract_address, "data": selector}, "latest"])))
            .collect();
        let results = self.client.batch_call::<String>(calls).await?.extract();
        results.try_into().map_err(|_| "Array conversion failed".into())
    }

    pub async fn get_fee_history(&self, blocks: u64, reward_percentiles: Vec<u64>) -> Result<EthereumFeeHistory, JsonRpcError> {
        let params = json!([format!("0x{:x}", blocks), "latest", reward_percentiles]);
        self.client.call("eth_feeHistory", params).await
    }

    pub async fn batch_token_balance_calls(&self, address: &str, contracts: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error + Sync + Send>> {
        let data = format!("0x70a08231000000000000000000000000{:0>40}", address.strip_prefix("0x").unwrap_or(address));
        let calls: Vec<(String, serde_json::Value)> = contracts
            .iter()
            .map(|x| ("eth_call".to_string(), json!([{"to": x, "data": &data}, "latest"])))
            .collect();
        Ok(self.client.batch_call::<String>(calls).await?.extract())
    }
}
