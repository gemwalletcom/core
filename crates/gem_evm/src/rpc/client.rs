use alloy_primitives::{Address, Bytes as AlloyBytes};
use alloy_rpc_client::{ClientBuilder, RpcClient};
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionRequest as AlloyTransactionRequest};
use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::str::FromStr;
use url::Url;

use super::model::{Block, TransactionReciept};
use primitives::{Chain, EVMChain};

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

pub struct EthereumClient {
    chain: EVMChain,
    client: RpcClient,
}

impl EthereumClient {
    pub fn new(chain: EVMChain, url_str: String) -> Self {
        let url = Url::parse(&url_str).expect("Invalid Ethereum node URL");
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

    pub async fn get_latest_block(&self) -> Result<i64> {
        let block_hex: String = self.client.request("eth_blockNumber", ()).await?;
        if !block_hex.starts_with("0x") {
            return Err(anyhow!("Invalid block number format: {}", block_hex));
        }
        Ok(i64::from_str_radix(&block_hex[2..], 16)?)
    }
}
