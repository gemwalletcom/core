use super::model::{Block, TransactionReciept};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use primitives::chain::Chain;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::error::Error;

pub const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
pub const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
pub const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";

pub struct EthereumClient {
    chain: Chain,
    client: HttpClient,
}

impl EthereumClient {
    pub fn new(chain: Chain, url: String) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(256 * 1024 * 1024) // 256MB
            .build(url)
            .unwrap();

        Self { chain, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn eth_call<T: DeserializeOwned>(&self, contract: &str, data: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let res = self
            .client
            .request(
                "eth_call",
                vec![
                    json!({
                        "to": contract,
                        "data": data,
                    }),
                    json!("latest"),
                ],
            )
            .await?;
        Ok(res)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .request("eth_getBlockByNumber", vec![json!(format!("0x{:x}", block_number)), json!(true)])
            .await?)
    }

    pub async fn get_block_reciepts(&self, block_number: i64) -> Result<Vec<TransactionReciept>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .request("eth_getBlockReceipts", vec![json!(format!("0x{:x}", block_number))])
            .await?)
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = self.client.request("eth_blockNumber", rpc_params![]).await?;
        Ok(i64::from_str_radix(&block[2..], 16)?)
    }
}
