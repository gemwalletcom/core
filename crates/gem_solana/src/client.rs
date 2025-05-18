use jsonrpsee::{
    core::{client::ClientT, params::BatchRequestBuilder, ClientError},
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::{error::Error, fmt::Debug, str::FromStr};

use crate::{
    metaplex::{decode_metadata, metadata::Metadata},
    model::{BlockTransactions, TokenAccountInfo, ValueData, ValueResult},
    pubkey::Pubkey,
};

pub struct SolanaClient {
    client: HttpClient,
}

impl SolanaClient {
    pub fn new(url: &str) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(100 * 1024 * 1024) // 100MB
            .build(url)
            .unwrap();

        Self { client }
    }
    pub async fn get_account_info<T: DeserializeOwned>(&self, account: &str, encoding: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let rpc_method = "getAccountInfo";
        let params = vec![
            json!(account),
            json!({
                "encoding": encoding,
                "commitment": "confirmed",
            }),
        ];
        let info: T = self.client.request(rpc_method, params).await?;
        Ok(info)
    }

    pub async fn get_account_info_batch<T: DeserializeOwned + Debug + Clone>(
        &self,
        accounts: Vec<String>,
        encoding: &str,
        batch: usize,
    ) -> Result<Vec<T>, Box<dyn Error + Send + Sync>> {
        let accounts_chunks: Vec<Vec<String>> = accounts.chunks(batch).map(|s| s.into()).collect();
        let mut results: Vec<T> = Vec::new();
        for accounts in accounts_chunks {
            let mut batch = BatchRequestBuilder::default();
            for account in accounts.iter() {
                let params = vec![
                    json!(account),
                    json!({
                        "encoding": encoding,
                        "commitment": "confirmed",
                    }),
                ];
                batch.insert("getAccountInfo", params)?;
            }

            let data = self
                .client
                .batch_request::<T>(batch)
                .await?
                .iter()
                .filter_map(|r| r.as_ref().ok())
                .cloned()
                .collect::<Vec<T>>();

            if data.len() != accounts.len() {
                return Err("Failed to get all transaction reciepts".into());
            }
            results.extend(data);
        }
        Ok(results)
    }

    pub async fn get_metaplex_data(&self, token_mint: &str) -> Result<Metadata, Box<dyn Error + Send + Sync>> {
        let pubkey = Pubkey::from_str(token_mint)?;
        let metadata_key = Metadata::find_pda(pubkey)
            .ok_or::<Box<dyn Error + Send + Sync>>("metadata program account not found".into())?
            .0
            .to_string();

        let result: ValueResult<Option<ValueData<Vec<String>>>> = self.get_account_info(&metadata_key, "base64").await?;
        let value = result.value.ok_or(anyhow::anyhow!("Failed to get metadata"))?;
        let meta = decode_metadata(&value.data[0]).map_err(|_| anyhow::anyhow!("Failed to decode metadata"))?;
        Ok(meta)
    }

    pub async fn get_slot(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.request("getSlot", rpc_params![]).await?)
    }

    pub async fn get_block(
        &self,
        slot: i64,
        encoding: Option<&str>,
        transaction_details: Option<&str>,
        rewards: Option<bool>,
        max_supported_transaction_version: Option<u8>,
    ) -> Result<BlockTransactions, ClientError> {
        let params = vec![
            json!(slot),
            json!({
                "encoding": encoding.unwrap_or("jsonParsed"),
                "transactionDetails": transaction_details.unwrap_or("full"),
                "rewards": rewards.unwrap_or(false),
                "maxSupportedTransactionVersion": max_supported_transaction_version.unwrap_or(0),
            }),
        ];
        self.client.request("getBlock", params).await
    }

    pub async fn get_token_accounts_by_owner(&self, owner: &str, program_id: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(owner),
            json!({
                "programId": program_id
            }),
            json!({
                "encoding": "jsonParsed"
            }),
        ];
        Ok(self.client.request("getTokenAccountsByOwner", params).await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::ResultTokenInfo;

    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct JsonRpcResult<T> {
        result: T,
    }

    #[test]
    fn test_decode_token_data() {
        let pyusd_file = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/pyusd_mint.json");
        let usdc_file = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/usdc_mint.json");

        let file = std::fs::File::open(pyusd_file).expect("file should open read only");
        let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
        let result: JsonRpcResult<ResultTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        let parsed_info = result.result.value.data.parsed.info;
        assert_eq!(parsed_info.decimals, 6);
        assert_eq!(parsed_info.mint_authority, "22mKJkKjGEQ3rampp5YKaSsaYZ52BUkcnUN6evXGsXzz");

        let file = std::fs::File::open(usdc_file).expect("file should open read only");
        let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
        let result: JsonRpcResult<ResultTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        let parsed_info = result.result.value.data.parsed.info;

        assert_eq!(parsed_info.decimals, 6);
        assert_eq!(parsed_info.mint_authority, "BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG");
    }
}
