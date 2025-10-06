use crate::models::{
    EpochInfo, InflationRate, ResultTokenInfo, Signature, TokenAccountInfo, ValueResult, VoteAccounts,
    balances::SolanaBalance,
    blockhash::SolanaBlockhashResult,
    prioritization_fee::SolanaPrioritizationFee,
    transaction::{BlockTransactions, SolanaTransaction},
};
use chain_traits::ChainProvider;
#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainTraits};
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use gem_jsonrpc::{client::JsonRpcClient as GenericJsonRpcClient, types::JsonRpcError};
use primitives::Chain;
use std::error::Error;

#[cfg(feature = "rpc")]
pub struct SolanaClient<C: Client + Clone> {
    client: GenericJsonRpcClient<C>,
    pub chain: Chain,
}

pub fn token_accounts_by_owner_params(owner: &str, program_id: &str) -> serde_json::Value {
    serde_json::json!([
        owner,
        {
            "programId": program_id
        },
        {
            "encoding": "jsonParsed"
        }
    ])
}

pub fn token_accounts_by_mint_params(owner: &str, mint: &str) -> serde_json::Value {
    serde_json::json!([
        owner,
        {
            "mint": mint
        },
        {
            "encoding": "jsonParsed"
        }
    ])
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

    pub async fn rpc_call<T>(&self, method: &str, params: serde_json::Value) -> Result<T, JsonRpcError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.client.call(method, params).await
    }

    pub async fn get_balance(&self, address: &str) -> Result<SolanaBalance, JsonRpcError> {
        self.rpc_call("getBalance", serde_json::json!([address])).await
    }

    pub async fn get_token_accounts_by_owner(&self, owner: &str, program_id: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, JsonRpcError> {
        let params = token_accounts_by_owner_params(owner, program_id);
        self.rpc_call("getTokenAccountsByOwner", params).await
    }

    pub async fn get_epoch_info(&self) -> Result<EpochInfo, JsonRpcError> {
        self.rpc_call("getEpochInfo", serde_json::json!([])).await
    }

    pub async fn get_token_accounts_by_mint(&self, owner: &str, mint: &str) -> Result<ValueResult<Vec<TokenAccountInfo>>, JsonRpcError> {
        let params = token_accounts_by_mint_params(owner, mint);
        self.rpc_call("getTokenAccountsByOwner", params).await
    }

    pub async fn get_transaction(&self, signature: &str) -> Result<SolanaTransaction, JsonRpcError> {
        let params = serde_json::json!([
            signature,
            {
                "maxSupportedTransactionVersion": 0
            }
        ]);
        self.rpc_call("getTransaction", params).await
    }

    pub async fn get_genesis_hash(&self) -> Result<String, JsonRpcError> {
        self.rpc_call("getGenesisHash", serde_json::json!([])).await
    }

    pub async fn get_slot(&self) -> Result<u64, JsonRpcError> {
        self.rpc_call("getSlot", serde_json::json!([])).await
    }

    pub async fn get_latest_blockhash(&self) -> Result<SolanaBlockhashResult, JsonRpcError> {
        self.rpc_call("getLatestBlockhash", serde_json::json!([])).await
    }

    pub async fn get_staking_balance(&self, address: &str) -> Result<Vec<TokenAccountInfo>, JsonRpcError> {
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

    pub async fn get_vote_accounts(&self) -> Result<VoteAccounts, JsonRpcError> {
        let params = serde_json::json!([{
            "keepUnstakedDelinquents": true,
            "commitment": "finalized"
        }]);
        self.rpc_call("getVoteAccounts", params).await
    }

    pub async fn get_inflation_rate(&self) -> Result<InflationRate, JsonRpcError> {
        self.rpc_call("getInflationRate", serde_json::json!([])).await
    }

    pub async fn send_transaction(&self, data: String, skip_preflight: Option<bool>) -> Result<String, JsonRpcError> {
        let mut params = serde_json::json!([
            data,
            {
                "encoding": "base64"
            }
        ]);

        if let Some(skip) = skip_preflight {
            params = serde_json::json!([
                data,
                {
                    "encoding": "base64",
                    "skipPreflight": skip
                }
            ]);
        }

        self.rpc_call("sendTransaction", params).await
    }

    pub async fn get_recent_prioritization_fees(&self) -> Result<Vec<SolanaPrioritizationFee>, JsonRpcError> {
        self.rpc_call("getRecentPrioritizationFees", serde_json::json!([])).await
    }

    pub async fn get_token_mint_info(&self, token_mint: &str) -> Result<ResultTokenInfo, JsonRpcError> {
        self.rpc_call(
            "getAccountInfo",
            serde_json::json!([
                token_mint,
                {
                    "encoding": "jsonParsed"
                }
            ]),
        )
        .await
    }

    pub async fn get_metaplex_metadata(&self, token_mint: &str) -> Result<crate::metaplex::metadata::Metadata, Box<dyn Error + Send + Sync>> {
        use crate::{
            metaplex::decode_metadata,
            metaplex::metadata::Metadata,
            models::{ValueData, ValueResult},
            pubkey::Pubkey,
        };
        use std::str::FromStr;

        let pubkey = Pubkey::from_str(token_mint)?;
        let metadata_key = Metadata::find_pda(pubkey)
            .ok_or::<Box<dyn Error + Send + Sync>>("metadata program account not found".into())?
            .0
            .to_string();

        let result: ValueResult<Option<ValueData<Vec<String>>>> = self
            .rpc_call(
                "getAccountInfo",
                serde_json::json!([
                    metadata_key,
                    {
                        "encoding": "base64"
                    }
                ]),
            )
            .await?;

        let value = result.value.ok_or("Failed to get metadata")?;
        let meta = decode_metadata(&value.data[0]).map_err(|_| "Failed to decode metadata")?;
        Ok(meta)
    }

    pub async fn get_block_transactions(&self, slot: u64) -> Result<BlockTransactions, JsonRpcError> {
        let params = serde_json::json!([
            slot,
            {
                "encoding": "json",
                "transactionDetails": "full",
                "rewards": false,
                "maxSupportedTransactionVersion": 0
            }
        ]);
        self.rpc_call("getBlock", params).await
    }

    pub async fn get_signatures_for_address(&self, address: &str, limit: usize) -> Result<Vec<Signature>, JsonRpcError> {
        let params = serde_json::json!([
            address,
            {
                "limit": limit,
                "commitment": "confirmed"
            }
        ]);
        self.rpc_call("getSignaturesForAddress", params).await
    }

    pub async fn get_transactions(&self, signatures: Vec<String>) -> Result<Vec<crate::models::BlockTransaction>, JsonRpcError> {
        let mut transactions = Vec::new();

        for signature in signatures {
            let params = serde_json::json!([
                signature,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
                }
            ]);

            if let Ok(tx) = self.rpc_call::<crate::models::BlockTransaction>("getTransaction", params).await {
                transactions.push(tx);
            }
        }

        Ok(transactions)
    }

    pub async fn get_token_accounts(
        &self,
        address: &str,
        token_mints: &[String],
    ) -> Result<Vec<ValueResult<Vec<TokenAccountInfo>>>, Box<dyn Error + Send + Sync>> {
        let calls: Vec<(String, serde_json::Value)> = token_mints
            .iter()
            .map(|mint| ("getTokenAccountsByOwner".to_string(), token_accounts_by_mint_params(address, mint)))
            .collect();

        let results = self.get_client().batch_call(calls).await?.extract();
        Ok(results)
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
impl<C: Client + Clone> ChainAddressStatus for SolanaClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> ChainTraits for SolanaClient<C> {}
impl<C: Client + Clone> ChainProvider for SolanaClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        Chain::Solana
    }
}

#[cfg(test)]
mod tests {
    use crate::models::ResultTokenInfo;
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
