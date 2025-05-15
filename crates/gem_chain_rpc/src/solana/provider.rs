use std::error::Error;

use async_trait::async_trait;
use jsonrpsee::core::ClientError;
use serde_json::json;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use gem_solana::metaplex::metadata::Metadata;
use primitives::{chain::Chain, Asset, AssetId, AssetType};

use super::{client::SolanaClient, mapper::SolanaMapper, model::BlockTransactions};

pub struct SolanaProvider {
    client: SolanaClient,
}

// Error codes from the Solana RPC
const CLEANUP_BLOCK_ERROR: i32 = -32001;
const MISSING_SLOT_ERROR: i32 = -32007;
const MISSING_OR_SKIPPED_SLOT_ERROR: i32 = -32009;
const NOT_AVAILABLE_SLOT_ERROR: i32 = -32004;

impl SolanaProvider {
    pub fn new(client: SolanaClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for SolanaProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(block_number),
            json!({
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0,
                "transactionDetails": "full",
                "rewards": false
            }),
        ];
        let block: Result<BlockTransactions, ClientError> = self.client.request_block(params).await;
        match block {
            Ok(block) => {
                let transactions = block
                    .transactions
                    .into_iter()
                    .flat_map(|x| SolanaMapper::map_transaction(self.client.get_chain(), &x, block_number))
                    .collect::<Vec<primitives::Transaction>>();
                Ok(transactions)
            }
            Err(err) => match err {
                ClientError::Call(err) => {
                    let errors = [MISSING_SLOT_ERROR, MISSING_OR_SKIPPED_SLOT_ERROR, NOT_AVAILABLE_SLOT_ERROR, CLEANUP_BLOCK_ERROR];
                    if errors.contains(&err.code()) {
                        return Ok(vec![]);
                    } else {
                        return Err(Box::new(err));
                    }
                }
                _ => return Err(Box::new(err)),
            },
        }
    }
}

#[async_trait]
impl ChainTokenDataProvider for SolanaProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        // Use the SolanaClient's get_account_info method with generic type parameter
        let token_info = self
            .client
            .get_account_info::<gem_solana::jsonrpc::SolanaParsedTokenInfo>(&token_id, "jsonParsed")
            .await?;
        let meta: Metadata = self.client.get_metaplex_data(&token_id).await?;
        let name = meta.data.name.trim_matches(char::from(0)).to_string();
        let symbol = meta.data.symbol.trim_matches(char::from(0)).to_string();
        let decimals = token_info.value.data.parsed.info.decimals;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            name,
            symbol,
            decimals,
            AssetType::SPL,
        ))
    }
}
