use async_trait::async_trait;
use jsonrpsee::core::ClientError;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider, ChainTransactionsProvider};

use gem_solana::{
    model::ResultTokenInfo,
    rpc::{client::SolanaClient, mapper::SolanaMapper},
    TOKEN_PROGRAM,
};
use primitives::{chain::Chain, Asset, AssetBalance, AssetId, Transaction};

const CLEANUP_BLOCK_ERROR: i32 = -32001;
const MISSING_SLOT_ERROR: i32 = -32007;
const MISSING_OR_SKIPPED_SLOT_ERROR: i32 = -32009;
const NOT_AVAILABLE_SLOT_ERROR: i32 = -32004;

pub struct SolanaProvider {
    client: SolanaClient,
}

impl SolanaProvider {
    pub fn new(client: SolanaClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainBlockProvider for SolanaProvider {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_slot().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number, Some("json"), Some("full"), Some(false), Some(0)).await;
        match block {
            Ok(block) => {
                let transactions = block
                    .transactions
                    .into_iter()
                    .filter_map(|x| SolanaMapper::map_transaction(&x, block_number))
                    .collect::<Vec<_>>();
                Ok(transactions)
            }
            Err(err) => match err {
                ClientError::Call(err) => {
                    let errors = [MISSING_SLOT_ERROR, MISSING_OR_SKIPPED_SLOT_ERROR, NOT_AVAILABLE_SLOT_ERROR, CLEANUP_BLOCK_ERROR];
                    if errors.contains(&err.code()) {
                        Ok(vec![])
                    } else {
                        Err(Box::new(err))
                    }
                }
                _ => Err(Box::new(err)),
            },
        }
    }
}

#[async_trait]
impl ChainAssetsProvider for SolanaProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let accounts = self.client.get_token_accounts_by_owner(&address, TOKEN_PROGRAM).await?.value;

        Ok(accounts
            .into_iter()
            .map(|x| AssetBalance {
                asset_id: AssetId::from_token(self.get_chain(), &x.account.data.parsed.info.mint),
                balance: x.account.data.parsed.info.token_amount.amount.to_string(),
            })
            .collect())
    }
}

#[async_trait]
impl ChainTokenDataProvider for SolanaProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let token_info = self.client.get_account_info::<ResultTokenInfo>(&token_id, "jsonParsed").await?.info();
        let meta = self.client.get_metaplex_data(&token_id).await?;
        SolanaMapper::map_token_data(self.get_chain(), token_id, &token_info, &meta)
    }
}

#[async_trait]
impl ChainTransactionsProvider for SolanaProvider {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}
