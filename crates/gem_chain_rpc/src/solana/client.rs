use chrono::Utc;
use jsonrpsee::{
    core::{client::ClientT, params::BatchRequestBuilder, ClientError},
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{error::Error, fmt::Debug, str::FromStr};

use super::model::{BlockTransaction, BlockTransactions, InstructionParsed};
use gem_solana::{
    jsonrpc::{AccountData, ValueResult},
    metaplex::{decode_metadata, metadata::Metadata},
    pubkey::Pubkey,
    TOKEN_PROGRAM, WSOL_TOKEN_ADDRESS,
};
use primitives::{chain::Chain, AssetId, Transaction, TransactionState, TransactionSwapMetadata, TransactionType};

pub struct SolanaClient {
    client: HttpClient,
}

pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";

impl SolanaClient {
    pub fn new(url: &str) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(100 * 1024 * 1024) // 100MB
            .build(url)
            .unwrap();

        Self { client }
    }

    pub fn map_transaction(&self, transaction: &BlockTransaction, block_number: i64) -> Option<primitives::Transaction> {
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
        let created_at = Utc::now();
        // system transfer
        if (account_keys.len() == 2 || account_keys.len() == 3) && account_keys.last()? == SYSTEM_PROGRAM_ID && signatures.len() == 1 {
            let from = account_keys.first()?.clone();
            let to = account_keys[account_keys.len() - 2].clone();

            let value = transaction.meta.pre_balances[0] - transaction.meta.post_balances[0] - fee;

            let transaction = Transaction::new(
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
                created_at,
            );
            return Some(transaction);
        }

        let pre_token_balances = transaction.meta.pre_token_balances.clone();
        let post_token_balances = transaction.meta.post_token_balances.clone();

        // SPL transfer. Limit to 7 accounts.
        if account_keys.contains(&TOKEN_PROGRAM.to_string())
            && account_keys.len() <= 7
            && (pre_token_balances.len() == 1 || pre_token_balances.len() == 2)
            && post_token_balances.len() == 2
        {
            let token_id = transaction.meta.pre_token_balances.first()?.mint.clone();
            let asset_id = AssetId {
                chain: self.get_chain(),
                token_id: Some(token_id),
            };

            let sender_account_index: i64 = if transaction.meta.pre_token_balances.len() == 1 {
                transaction.meta.pre_token_balances.first()?.account_index
            } else if pre_token_balances.first()?.get_amount() >= post_token_balances.first()?.get_amount() {
                pre_token_balances.first()?.account_index
            } else {
                post_token_balances.last()?.account_index
            };
            let recipient_account_index = post_token_balances.iter().find(|b| b.account_index != sender_account_index)?.account_index;

            let sender = transaction.meta.get_post_token_balance(sender_account_index)?;
            let recipient = transaction.meta.get_post_token_balance(recipient_account_index)?;
            let from_value = transaction.meta.get_pre_token_balance(sender_account_index)?.get_amount();
            let to_value = transaction.meta.get_post_token_balance(sender_account_index)?.get_amount();

            if to_value > from_value {
                return None;
            }
            let value = from_value - to_value;

            let from = sender.owner.clone();
            let to = recipient.owner.clone();

            let transaction = Transaction::new(
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
                created_at,
            );
            return Some(transaction);
        }

        if account_keys.contains(&JUPITER_PROGRAM_ID.to_string()) {
            for inner_transaction in transaction.meta.inner_instructions.clone() {
                let instructions = inner_transaction
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

                    let from_address = input.info.authority?;
                    let to_address = from_address.clone();

                    let from_asset = self.asset_id_from_program(input.info.mint?);
                    let to_asset = self.asset_id_from_program(output.info.mint?);
                    let from_value = input.info.token_amount?.amount.to_string();
                    let to_value = output.info.token_amount?.amount.to_string();

                    let swap = TransactionSwapMetadata {
                        from_asset: from_asset.clone(),
                        from_value: from_value.clone(),
                        to_asset: to_asset.clone(),
                        to_value: to_value.clone(),
                        provider: None,
                    };
                    let asset_id = from_asset.clone();

                    let transaction = Transaction::new(
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
                        created_at,
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

        let result: ValueResult<Option<AccountData>> = self.get_account_info(&metadata_key, "base64").await?;
        let value = result.value.ok_or(anyhow::anyhow!("Failed to get metadata"))?;
        let meta = decode_metadata(&value.data[0]).map_err(|_| anyhow::anyhow!("Failed to decode metadata"))?;
        Ok(meta)
    }
    
    pub fn get_chain(&self) -> Chain {
        Chain::Solana
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: i64 = self.client.request("getSlot", rpc_params![]).await?;
        Ok(block)
    }
    
    pub async fn request_block(&self, params: Vec<Value>) -> Result<BlockTransactions, ClientError> {
        self.client.request("getBlock", params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::jsonrpc::JsonRpcResult;
    use serde::Deserialize;
    
    #[derive(Debug, Deserialize)]
    struct SolanaParsedTokenInfo {
        result: ValueResult<ParsedTokenAccount>,
    }
    
    #[derive(Debug, Deserialize)]
    struct ParsedTokenAccount {
        value: TokenAccount,
    }
    
    #[derive(Debug, Deserialize)]
    struct TokenAccount {
        data: TokenAccountData,
    }
    
    #[derive(Debug, Deserialize)]
    struct TokenAccountData {
        parsed: TokenAccountParsed,
    }
    
    #[derive(Debug, Deserialize)]
    struct TokenAccountParsed {
        info: TokenInfo,
    }
    
    #[derive(Debug, Deserialize)]
    struct TokenInfo {
        decimals: u8,
        mint_authority: String,
    }

    #[test]
    fn test_decode_token_data() {
        let pyusd_file = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/solana/pyusd_mint.json");
        let usdc_file = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/solana/usdc_mint.json");

        let file = std::fs::File::open(pyusd_file).expect("file should open read only");
        let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
        let result: JsonRpcResult<SolanaParsedTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        let parsed_info = result.result.value.data.parsed.info;

        assert_eq!(parsed_info.decimals, 6);
        assert_eq!(parsed_info.mint_authority, "22mKJkKjGEQ3rampp5YKaSsaYZ52BUkcnUN6evXGsXzz");

        let file = std::fs::File::open(usdc_file).expect("file should open read only");
        let json: serde_json::Value = serde_json::from_reader(file).expect("file should be proper JSON");
        let result: JsonRpcResult<SolanaParsedTokenInfo> = serde_json::from_value(json).expect("Decoded into ParsedTokenInfo");
        let parsed_info = result.result.value.data.parsed.info;

        assert_eq!(parsed_info.decimals, 6);
        assert_eq!(parsed_info.mint_authority, "BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG");
    }
}
