use std::error::Error;

use crate::{ChainProvider, solana::model::BlockTransactions};
use async_trait::async_trait;
use chrono::Utc;
use ethers::providers::{JsonRpcClient, Http, RetryClientBuilder, RetryClient, RetryClientError, RpcError};
use primitives::{chain::Chain, Transaction, TransactionType, TransactionState, AssetId};
use reqwest::Url;
use serde_json::json;

use super::model::BlockTransaction;

pub struct SolanaClient {
    client: RetryClient<Http>,
}

const MISSING_SLOT_ERROR: i64 = -32007;
const NOT_AVAILABLE_SLOT_ERROR: i64 = -32004;
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

impl SolanaClient {
    pub fn new(url: String) -> Self {
        let provider = Http::new(Url::parse(url.as_str()).unwrap());
        let client = RetryClientBuilder::default()
            .build(provider, Box::<ethers::providers::HttpRateLimitRetryPolicy>::default());
        
        Self {
            client,
        }
    }

    fn map_transaction(&self, transaction: &BlockTransaction, block_number: i64) -> Option<primitives::Transaction> {
        let account_keys = transaction.transaction.message.account_keys.clone();
        let signatures = transaction.transaction.signatures.clone();
        let log_messages = transaction.meta.log_messages.clone().unwrap_or_default();
        let hash = transaction.transaction.signatures.first()?.to_string();
        let chain = self.get_chain();
        let fee = transaction.meta.fee;
        let sequence = 0.to_string();
        let state = TransactionState::Confirmed;
        let fee_asset_id = chain.as_asset_id();

        // system transfer
        if (account_keys.len() == 2 || account_keys.len() == 3) && account_keys.last().unwrap() == SYSTEM_PROGRAM_ID && signatures.len() == 1 && log_messages.len() == 2 {    
            let from = account_keys.first()?.clone();
            let to = account_keys[account_keys.len() - 2].clone();
            let value = transaction.meta.pre_balances[0] - transaction.meta.post_balances[0] - fee;  
    
            let transaction = primitives::Transaction::new(
                hash,
                chain.as_asset_id(), 
                from, 
                to, 
                None, 
                TransactionType::Transfer, 
                state, 
                block_number.to_string(), 
                sequence, 
                fee.to_string(), 
                fee_asset_id, 
                value.to_string(), 
                None,
                Utc::now(),
            );
            return Some(transaction);
        }
        
        let pre_token_balances = transaction.meta.pre_token_balances.clone();
        let post_token_balances = transaction.meta.post_token_balances.clone();

        // SLP transfer. Limit to 7 accounts.
        if account_keys.contains(&TOKEN_PROGRAM_ID.to_string()) && account_keys.len() <= 7 && pre_token_balances.len() > 0 && post_token_balances.len() == 2 {
            let token_id = transaction.meta.pre_token_balances.first()?.mint.clone();
            let asset_id = AssetId { chain: self.get_chain(), token_id: Some(token_id) };

            let sender_account_index: i64 = if transaction.meta.pre_token_balances.len() == 1 {
                transaction.meta.pre_token_balances.first()?.account_index.clone()
            } else {
                if pre_token_balances.first()?.get_amount() >= post_token_balances.first()?.get_amount() {
                    pre_token_balances.first()?.account_index.clone()
                } else {
                    post_token_balances.last()?.account_index.clone()
                }
            };
            let recipient_account_index = post_token_balances.iter().find(|b| b.account_index != sender_account_index)?.account_index.clone();

            let sender = transaction.meta.get_post_token_balance(sender_account_index)?;
            let recipient = transaction.meta.get_post_token_balance(recipient_account_index)?;

            let value = transaction.meta.get_pre_token_balance(sender_account_index)?.get_amount() - transaction.meta.get_post_token_balance(sender_account_index)?.get_amount();

            let from = sender.owner.clone();
            let to = recipient.owner.clone();

            let transaction = primitives::Transaction::new(
                hash,
                asset_id, 
                from, 
                to, 
                None, 
                TransactionType::Transfer, 
                state, 
                block_number.to_string(), 
                sequence, 
                fee.to_string(), 
                fee_asset_id, 
                value.to_string(), 
                None,
                Utc::now(),
            );
            return Some(transaction);
        }

        None 
    }
}

#[async_trait]
impl ChainProvider for SolanaClient {

    fn get_chain(&self) -> Chain {
        Chain::Solana
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: i64 = JsonRpcClient::request(&self.client, "getSlot", ()).await?;
        Ok(block)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(block_number),
            json!({
                "encoding": "json",
                "maxSupportedTransactionVersion": 0,
                "transactionDetails": "full",
                "rewards": false
            })
        ];
        let block: Result<BlockTransactions, RetryClientError> = JsonRpcClient::request(&self.client, "getBlock", params).await;
        match block {
            Ok(block) => {
                let transactions = block.transactions
                    .into_iter()
                    .flat_map(|x| self.map_transaction(&x, block_number))
                    .collect::<Vec<primitives::Transaction>>();
                Ok(transactions)
            },
            Err(err) => {
                match err {
                    RetryClientError::ProviderError(err) => {
                        if let Some(json_error) =  err.as_error_response() {
                            if [MISSING_SLOT_ERROR, NOT_AVAILABLE_SLOT_ERROR].contains(&json_error.code) {
                                return Ok(vec![])
                            }
                        };
                        return Err(Box::new(err))
                    },
                    _ => {
                        return Err(Box::new(err))
                    }
                }
            }
        } 
    }
}