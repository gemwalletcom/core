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
    model::{BlockTransaction, BlockTransactions, Signature, TokenAccountInfo, ValueData, ValueResult},
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
        Ok(self.client.request(rpc_method, params).await?)
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
                "encoding": encoding.unwrap_or("json"),
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

    pub async fn get_transaction(&self, signature: &str) -> Result<Vec<Signature>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(signature),
            json!({
                "maxSupportedTransactionVersion": 0
            }),
        ];
        Ok(self.client.request("getTransaction", params).await?)
    }

    pub async fn get_transactions(&self, signatures: Vec<String>) -> Result<Vec<BlockTransaction>, Box<dyn Error + Send + Sync>> {
        let mut batch = BatchRequestBuilder::default();

        for signature in &signatures {
            batch.insert(
                "getTransaction",
                vec![
                    json!(signature),
                    json!({
                        "maxSupportedTransactionVersion": 0,
                        "transactionDetails": "full",
                    }),
                ],
            )?;
        }

        let data = self
            .client
            .batch_request::<BlockTransaction>(batch)
            .await?
            .iter()
            .map(|x| x.as_ref().unwrap().clone())
            .collect();

        Ok(data)
    }

    pub async fn get_signatures_for_address(&self, address: &str, limit: u64) -> Result<Vec<Signature>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(address),
            json!({
                "limit": limit
            }),
        ];
        Ok(self.client.request("getSignaturesForAddress", params).await?)
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
        let file = include_str!("../../testdata/pyusd_mint.json");
        let json: serde_json::Value = serde_json::from_str(file).expect("file should be proper JSON");
        let result: JsonRpcResult<ResultTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        let parsed_info = result.result.value.data.parsed.info;

        assert_eq!(parsed_info.decimals, 6);

        let file = include_str!("../../testdata/usdc_mint.json");
        let json: serde_json::Value = serde_json::from_str(file).expect("file should be proper JSON");
        let result: JsonRpcResult<ResultTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        let parsed_info = result.result.value.data.parsed.info;

        assert_eq!(parsed_info.decimals, 6);
    }
}
