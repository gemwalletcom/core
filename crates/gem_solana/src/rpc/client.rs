use crate::{
    model::{TokenAccountInfo, ValueResult, EpochInfo},
    models::balances::SolanaBalance,
};
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use gem_jsonrpc::client::JsonRpcClient as GenericJsonRpcClient;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTraits, ChainStaking, ChainTransactions, ChainState, ChainAccount, ChainPerpetual, ChainToken};
use std::error::Error;
use primitives::Chain;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
pub struct SolanaClient {
    client: JsonRpcClient,
}

#[cfg(feature = "rpc")]
pub struct SolanaClient<C: Client + Clone> {
    client: GenericJsonRpcClient<C>,
    pub chain: Chain,
}

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
impl SolanaClient {
    pub fn new(url: &str) -> Self {
        let client = JsonRpcClient::new_reqwest(url.to_string());

        Self { client }
    }
    pub async fn get_account_info<T: serde::de::DeserializeOwned>(&self, account: &str, encoding: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let rpc_method = "getAccountInfo";
        let params = vec![
            serde_json::json!(account),
            serde_json::json!({
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
                        serde_json::json!({
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
        Ok(self.client.call("getSlot", serde_json::json!([])).await?)
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
            serde_json::json!(slot),
            serde_json::json!({
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
            serde_json::json!(signature),
            serde_json::json!({
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
                        serde_json::json!(signature),
                        serde_json::json!({
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
            serde_json::json!(address),
            serde_json::json!({
                "limit": limit
            }),
        ];
        Ok(self.client.call("getSignaturesForAddress", params).await?)
    }

    pub async fn get_token_accounts_by_owner(&self, owner: &str, program_id: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            serde_json::json!(owner),
            serde_json::json!({
                "programId": program_id
            }),
            serde_json::json!({
                "encoding": "jsonParsed"
            }),
        ];
        Ok(self.client.call("getTokenAccountsByOwner", params).await?)
    }

    pub async fn get_vote_accounts(&self) -> Result<VoteAccounts, Box<dyn Error + Send + Sync>> {
        let params = vec![serde_json::json!({
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
            serde_json::json!("Config1111111111111111111111111111111111111"),
            serde_json::json!({
                "encoding": "jsonParsed"
            }),
        ];
        Ok(self.client.call("getProgramAccounts", params).await?)
    }

    pub async fn get_balance(&self, address: &str) -> Result<SolanaBalance, Box<dyn Error + Send + Sync>> {
        let params = vec![serde_json::json!(address)];
        Ok(self.client.call("getBalance", params).await?)
    }

    pub async fn get_stake_accounts(&self, address: &str) -> Result<Vec<TokenAccountInfo>, Box<dyn Error + Send + Sync>> {
        let stake_program_id = "Stake11111111111111111111111111111111111111";
        let params = vec![
            serde_json::json!(stake_program_id),
            serde_json::json!({
                "encoding": "jsonParsed",
                "filters": [
                    {
                        "memcmp": {
                            "offset": 12,
                            "bytes": address
                        }
                    }
                ]
            }),
        ];
        Ok(self.client.call("getProgramAccounts", params).await?)
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> SolanaClient<C> {
    pub fn new(client: GenericJsonRpcClient<C>) -> Self {
        Self { client, chain: Chain::Solana }
    }

    pub fn get_client(&self) -> &GenericJsonRpcClient<C> {
        &self.client
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn rpc_call<T>(&self, method: &str, params: serde_json::Value) -> Result<T, Box<dyn Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned,
    {
        Ok(self.client.call(method, params).await?)
    }

    pub async fn get_balance(&self, address: &str) -> Result<SolanaBalance, Box<dyn Error + Send + Sync>> {
        self.rpc_call("getBalance", serde_json::json!([address])).await
    }

    pub async fn get_token_accounts_by_owner(&self, owner: &str, program_id: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([
            owner,
            {
                "programId": program_id
            },
            {
                "encoding": "jsonParsed"
            }
        ]);
        self.rpc_call("getTokenAccountsByOwner", params).await
    }

    pub async fn get_epoch_info(&self) -> Result<EpochInfo, Box<dyn Error + Send + Sync>> {
        self.rpc_call("getEpochInfo", serde_json::json!([])).await
    }

    pub async fn get_token_accounts_by_mint(&self, owner: &str, mint: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([
            owner,
            {
                "mint": mint
            },
            {
                "encoding": "jsonParsed"
            }
        ]);
        self.rpc_call("getTokenAccountsByOwner", params).await
    }

    pub async fn get_transaction(&self, signature: &str) -> Result<crate::models::transaction::SolanaTransaction, Box<dyn Error + Send + Sync>> {
        let params = serde_json::json!([
            signature,
            {
                "maxSupportedTransactionVersion": 0
            }
        ]);
        self.rpc_call("getTransaction", params).await
    }

    pub async fn get_genesis_hash(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.rpc_call("getGenesisHash", serde_json::json!([])).await
    }

    pub async fn get_slot(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        self.rpc_call("getSlot", serde_json::json!([])).await
    }

    pub async fn get_latest_blockhash(&self) -> Result<crate::models::blockhash::SolanaBlockhashResult, Box<dyn Error + Send + Sync>> {
        self.rpc_call("getLatestBlockhash", serde_json::json!([])).await
    }

    pub async fn get_staking_balance(&self, address: &str) -> Result<Vec<TokenAccountInfo>, Box<dyn Error + Send + Sync>> {
        let stake_program_id = "Stake11111111111111111111111111111111111111";
        let params = serde_json::json!([
            stake_program_id,
            {
                "encoding": "jsonParsed",
                "filters": [
                    {
                        "memcmp": {
                            "offset": 12,
                            "bytes": address
                        }
                    }
                ]
            }
        ]);
        
        let stake_accounts: Vec<TokenAccountInfo> = self.rpc_call("getProgramAccounts", params).await?;
        Ok(stake_accounts)
    }

}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl<C: Client + Clone> ChainStaking for SolanaClient<C> {}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl<C: Client + Clone> ChainTransactions for SolanaClient<C> {
    async fn transaction_broadcast(&self, _data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        Err("Transaction broadcast not implemented for Solana".into())
    }
    
    async fn get_transaction_status(&self, request: primitives::TransactionStateRequest) -> Result<primitives::TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;
        
        if transaction.slot > 0 {
            if transaction.meta.err.is_some() {
                Ok(primitives::TransactionUpdate {
                    state: primitives::TransactionState::Failed,
                    changes: vec![],
                })
            } else {
                Ok(primitives::TransactionUpdate {
                    state: primitives::TransactionState::Confirmed,
                    changes: vec![],
                })
            }
        } else {
            Ok(primitives::TransactionUpdate {
                state: primitives::TransactionState::Pending,
                changes: vec![],
            })
        }
    }
}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl<C: Client + Clone> ChainState for SolanaClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.get_genesis_hash().await
    }
    
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.get_slot().await
    }
    
    async fn get_fee_rates(&self) -> Result<Vec<primitives::FeePriorityValue>, Box<dyn Error + Sync + Send>> {
        Err("Fee rates not implemented for Solana".into())
    }
}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl<C: Client + Clone> ChainAccount for SolanaClient<C> {}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl<C: Client + Clone> ChainPerpetual for SolanaClient<C> {}

#[cfg(feature = "rpc")]
#[async_trait::async_trait]
impl<C: Client + Clone> ChainToken for SolanaClient<C> {
    async fn get_token_data(&self, _token_id: String) -> Result<primitives::Asset, Box<dyn Error + Sync + Send>> {
        Err("Token data not implemented for Solana".into())
    }

    fn get_is_token_address(&self, token_id: &str) -> bool {
        token_id.len() >= 40 && token_id.len() <= 60 && bs58::decode(token_id).into_vec().is_ok()
    }
}


#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainTraits for SolanaClient<C> {}

#[cfg(test)]
mod tests {
    use crate::model::ResultTokenInfo;
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
