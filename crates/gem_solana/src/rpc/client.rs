use crate::{
    metaplex::{decode_metadata, metadata::Metadata},
    model::{
        BlockTransaction, BlockTransactions, EpochInfo, InflationRate, Signature, TokenAccountInfo, ValidatorConfig, ValueData, ValueResult, VoteAccounts,
    },
    pubkey::Pubkey,
};
#[cfg(feature = "reqwest")]
use gem_jsonrpc::{
    types::{JsonRpcError, JsonRpcRequest, JsonRpcResult},
    JsonRpcClient,
};

#[cfg(not(feature = "reqwest"))]
use gem_jsonrpc::{
    types::{JsonRpcError, JsonRpcRequest, JsonRpcResult},
};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::{error::Error, fmt::Debug, str::FromStr};

#[cfg(feature = "reqwest")]
pub struct SolanaClient {
    client: JsonRpcClient,
}

#[cfg(feature = "reqwest")]
impl SolanaClient {
    pub fn new(url: &str) -> Self {
        let client = JsonRpcClient::new_reqwest(url.to_string());

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
        let info: T = self.client.call(rpc_method, params).await?;
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
            let calls = accounts
                .iter()
                .map(|account| {
                    (
                        "getAccountInfo".into(),
                        json!({
                            "params": [
                                account,
                                {
                                    "encoding": encoding,
                                    "commitment": "confirmed",
                                }
                            ],
                        }),
                    )
                })
                .collect::<Vec<_>>();

            let data = self
                .client
                .batch_call::<T>(calls)
                .await?
                .into_iter()
                .filter_map(|rpc_result| match rpc_result {
                    JsonRpcResult::Value(value) => Some(value.result),
                    JsonRpcResult::Error(e) => {
                        eprintln!("Error fetching account info for {}: {:?}", accounts.join(", "), e);
                        None
                    }
                })
                .collect::<Vec<T>>();

            if data.len() != accounts.len() {
                return Err("Failed to get all transaction receipts".into());
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
        Ok(self.client.call("getSlot", json!([])).await?)
    }

    pub async fn get_block(
        &self,
        slot: i64,
        encoding: Option<&str>,
        transaction_details: Option<&str>,
        rewards: Option<bool>,
        max_supported_transaction_version: Option<u8>,
    ) -> Result<BlockTransactions, JsonRpcError> {
        let params = vec![
            json!(slot),
            json!({
                "encoding": encoding.unwrap_or("json"),
                "transactionDetails": transaction_details.unwrap_or("full"),
                "rewards": rewards.unwrap_or(false),
                "maxSupportedTransactionVersion": max_supported_transaction_version.unwrap_or(0),
            }),
        ];
        self.client.call("getBlock", params).await
    }

    pub async fn get_transaction(&self, signature: &str) -> Result<Vec<Signature>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(signature),
            json!({
                "maxSupportedTransactionVersion": 0
            }),
        ];
        Ok(self.client.call("getTransaction", params).await?)
    }

    pub async fn get_transactions(&self, signatures: Vec<String>) -> Result<Vec<BlockTransaction>, Box<dyn Error + Send + Sync>> {
        let requests = signatures
            .iter()
            .enumerate()
            .map(|(index, signature)| {
                JsonRpcRequest::new(
                    index as u64 + 1,
                    "getTransaction",
                    vec![
                        json!(signature),
                        json!({
                            "maxSupportedTransactionVersion": 0,
                            "transactionDetails": "full",
                        }),
                    ]
                    .into(),
                )
            })
            .collect::<Vec<_>>();
        let results = self.client.batch_request::<BlockTransaction>(requests).await?;
        Ok(results.into_iter().flat_map(|x| x.take().ok()).collect())
    }

    pub async fn get_signatures_for_address(&self, address: &str, limit: u64) -> Result<Vec<Signature>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(address),
            json!({
                "limit": limit
            }),
        ];
        Ok(self.client.call("getSignaturesForAddress", params).await?)
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
        Ok(self.client.call("getTokenAccountsByOwner", params).await?)
    }

    pub async fn get_vote_accounts(&self) -> Result<VoteAccounts, Box<dyn Error + Send + Sync>> {
        let params = vec![json!({
            "keepUnstakedDelinquents": true,
            "commitment": "finalized"
        })];
        Ok(self.client.call("getVoteAccounts", params).await?)
    }

    pub async fn get_inflation_rate(&self) -> Result<InflationRate, Box<dyn Error + Send + Sync>> {
        let params: Vec<serde_json::Value> = vec![];
        Ok(self.client.call("getInflationRate", params).await?)
    }

    pub async fn get_epoch_info(&self) -> Result<EpochInfo, Box<dyn Error + Send + Sync>> {
        let params: Vec<serde_json::Value> = vec![];
        Ok(self.client.call("getEpochInfo", params).await?)
    }

    pub async fn get_validator_configs(&self) -> Result<Vec<ValidatorConfig>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!("Config1111111111111111111111111111111111111"),
            json!({
                "encoding": "jsonParsed"
            }),
        ];
        Ok(self.client.call("getProgramAccounts", params).await?)
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
