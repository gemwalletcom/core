use alloy_primitives::{Address, Bytes as AlloyBytes};
use alloy_rpc_client::RpcClient;
use alloy_rpc_types::{BlockId, BlockNumberOrTag, TransactionRequest as AlloyTransactionRequest};
use alloy_transport_http::reqwest::Client as ReqwestClient;
use alloy_transport_http::Http;
use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::str::FromStr;
use url::Url;

use super::model::{Block, TransactionReciept};
use primitives::chain::Chain;

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

pub struct EthereumClient {
    chain: Chain,
    client: RpcClient,
}

impl EthereumClient {
    pub fn new(chain: Chain, url_str: String) -> Self {
        let url = Url::parse(&url_str).expect("Invalid Ethereum node URL");
        let reqwest_client = ReqwestClient::new();
        let http_transport = Http::with_client(reqwest_client, url);
        let client = RpcClient::new(http_transport, true);
        Self { chain, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
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

        // Attempt to deserialize from the returned bytes.
        // If T is String, it usually expects a 0x-prefixed hex string.
        // If T is a struct, it expects JSON that matches the struct.
        if TypeId::of::<T>() == TypeId::of::<String>() {
            // Assuming the string is hex-encoded bytes
            Ok(serde_json::from_value(serde_json::Value::String(result_bytes.to_string()))?)
        } else {
            serde_json::from_slice(&result_bytes).map_err(|e| anyhow!(e))
        }
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block> {
        let block_id = BlockId::Number(BlockNumberOrTag::Number(block_number as u64));
        let include_txs = true;
        let params = (block_id, include_txs);

        let block: Option<Block> = self.client.request("eth_getBlockByNumber", params).await?;
        block.ok_or_else(|| anyhow!("Block not found or null response for block number: {}", block_number))
    }

    pub async fn get_block_receipts(&self, block_number: i64) -> Result<Vec<TransactionReciept>> {
        let block_id = BlockId::Number(BlockNumberOrTag::Number(block_number as u64));
        let params = (block_id,);

        let receipts: Vec<TransactionReciept> = self.client.request("eth_getBlockReceipts", params).await?;
        Ok(receipts)
    }

    pub async fn get_latest_block(&self) -> Result<i64> {
        let block_hex: String = self.client.request("eth_blockNumber", ()).await?;
        if !block_hex.starts_with("0x") {
            return Err(anyhow!("Invalid block number format: {}", block_hex));
        }
        Ok(i64::from_str_radix(&block_hex[2..], 16)?)
    }
}
