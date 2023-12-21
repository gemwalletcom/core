use std::error::Error;

use crate::{
    solana::model::{BlockTransactions, InstructionParsed},
    ChainProvider,
};
use async_trait::async_trait;
use chrono::Utc;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use primitives::{
    chain::Chain, AssetId, Transaction, TransactionState, TransactionSwapMetadata, TransactionType,
};

use serde_json::json;

use super::{model::BlockTransaction, WSOL_TOKEN_ADDRESS};

pub struct SolanaClient {
    client: HttpClient,
}

const CLEANUP_BLOCK_ERROR: i32 = -32001;
const MISSING_SLOT_ERROR: i32 = -32007;
const NOT_AVAILABLE_SLOT_ERROR: i32 = -32004;
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

impl SolanaClient {
    pub fn new(url: String) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(100 * 1024 * 1024) // 100MB
            .build(url)
            .unwrap();

        Self { client }
    }

    fn map_transaction(
        &self,
        transaction: &BlockTransaction,
        block_number: i64,
    ) -> Option<primitives::Transaction> {
        let account_keys = transaction
            .transaction
            .message
            .account_keys
            .clone()
            .into_iter()
            .map(|x| x.pubkey)
            .collect::<Vec<String>>();
        let signatures = transaction.transaction.signatures.clone();
        let hash = transaction.transaction.signatures.first()?.to_string();
        let chain = self.get_chain();
        let fee = transaction.meta.fee;
        let sequence = 0.to_string();
        let state = TransactionState::Confirmed;
        let fee_asset_id = chain.as_asset_id();
        // system transfer
        if (account_keys.len() == 2 || account_keys.len() == 3)
            && account_keys.last()? == SYSTEM_PROGRAM_ID
            && signatures.len() == 1
        {
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
                None,
                Utc::now(),
            );
            return Some(transaction);
        }

        let pre_token_balances = transaction.meta.pre_token_balances.clone();
        let post_token_balances = transaction.meta.post_token_balances.clone();

        // SLP transfer. Limit to 7 accounts.
        if account_keys.contains(&TOKEN_PROGRAM_ID.to_string())
            && account_keys.len() <= 7
            && (pre_token_balances.len() == 1 || pre_token_balances.len() == 2)
            && post_token_balances.len() == 2
        {
            println!("tx: {}", hash.clone());

            let token_id = transaction.meta.pre_token_balances.first()?.mint.clone();
            let asset_id = AssetId {
                chain: self.get_chain(),
                token_id: Some(token_id),
            };

            let sender_account_index: i64 = if transaction.meta.pre_token_balances.len() == 1 {
                transaction.meta.pre_token_balances.first()?.account_index
            } else if pre_token_balances.first()?.get_amount()
                >= post_token_balances.first()?.get_amount()
            {
                pre_token_balances.first()?.account_index
            } else {
                post_token_balances.last()?.account_index
            };
            let recipient_account_index = post_token_balances
                .iter()
                .find(|b| b.account_index != sender_account_index)?
                .account_index;

            let sender = transaction
                .meta
                .get_post_token_balance(sender_account_index)?;
            let recipient = transaction
                .meta
                .get_post_token_balance(recipient_account_index)?;
            let from_value = transaction
                .meta
                .get_pre_token_balance(sender_account_index)?
                .get_amount();
            let to_value = transaction
                .meta
                .get_post_token_balance(sender_account_index)?
                .get_amount();

            if to_value > from_value {
                return None;
            }
            let value = from_value - to_value;

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
                None,
                Utc::now(),
            );
            return Some(transaction);
        }

        if account_keys.contains(&JUPITER_PROGRAM_ID.to_string()) {
            for innter_transaction in transaction.meta.inner_instructions.clone() {
                let instructions = innter_transaction
                    .instructions
                    .clone()
                    .into_iter()
                    .flat_map(|x| {
                        if let Some(value) = x.parsed {
                            return Some(value);
                        }
                        None
                    })
                    .collect::<Vec<InstructionParsed>>();

                let transfer_instructions = instructions
                    .into_iter()
                    .filter(|x| x.instruction_type == "transferChecked")
                    .collect::<Vec<InstructionParsed>>();

                // 1 - input, 2 - referral, 3 destination
                if transfer_instructions.len() == 3 {
                    let input = transfer_instructions.first()?.clone();
                    let output = transfer_instructions.last()?.clone();

                    let from_address = input.info.authority.unwrap();
                    let to_address = from_address.clone();

                    let from_asset = self.asset_id_from_program(input.info.mint.unwrap());
                    let to_asset = self.asset_id_from_program(output.info.mint.unwrap());
                    let from_value = input.info.token_amount.unwrap().amount.value.to_string();
                    let to_value = output.info.token_amount.unwrap().amount.value.to_string();

                    let swap = TransactionSwapMetadata {
                        from_asset: from_asset.clone(),
                        from_value: from_value.clone(),
                        to_asset: to_asset.clone(),
                        to_value: to_value.clone(),
                    };
                    let asset_id = from_asset.clone();

                    let transaction = primitives::Transaction::new(
                        hash.clone(),
                        asset_id,
                        from_address,
                        to_address,
                        Some(JUPITER_PROGRAM_ID.to_string()),
                        TransactionType::Swap,
                        state,
                        block_number.to_string(),
                        sequence,
                        fee.to_string(),
                        chain.as_asset_id(),
                        from_value.clone().to_string(),
                        None,
                        serde_json::to_value(swap).ok(),
                        Utc::now(),
                    );
                    return Some(transaction);
                }
            }
        }

        None
    }

    fn asset_id_from_program(&self, program_id: String) -> AssetId {
        if program_id == WSOL_TOKEN_ADDRESS {
            return self.get_chain().as_asset_id();
        }
        AssetId {
            chain: self.get_chain(),
            token_id: Some(program_id),
        }
    }
}

#[async_trait]
impl ChainProvider for SolanaClient {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: i64 = self.client.request("getSlot", rpc_params![]).await?;
        Ok(block)
    }

    async fn get_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let params = vec![
            json!(block_number),
            json!({
                "encoding": "jsonParsed",
                "maxSupportedTransactionVersion": 0,
                "transactionDetails": "full",
                "rewards": false
            }),
        ];
        let block: Result<BlockTransactions, jsonrpsee::core::ClientError> =
            self.client.request("getBlock", params).await;
        match block {
            Ok(block) => {
                let transactions = block
                    .transactions
                    .into_iter()
                    .flat_map(|x| self.map_transaction(&x, block_number))
                    .collect::<Vec<primitives::Transaction>>();
                Ok(transactions)
            }
            Err(err) => match err {
                jsonrpsee::core::ClientError::Call(err) => {
                    let errors = [
                        MISSING_SLOT_ERROR,
                        NOT_AVAILABLE_SLOT_ERROR,
                        CLEANUP_BLOCK_ERROR,
                    ];
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
