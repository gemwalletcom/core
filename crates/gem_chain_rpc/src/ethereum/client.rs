use super::model::{Block, Transaction, TransactionReciept};
use crate::ethereum::erc20;
use crate::{ChainBlockProvider, ChainTokenDataProvider};
use alloy_primitives::hex;
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use chrono::DateTime;
use gem_evm::ethereum_address_checksum;
use hex::FromHex;
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use num_bigint::BigUint;
use num_traits::Num;
use primitives::{chain::Chain, Asset, AssetId, TransactionState, TransactionSwapMetadata, TransactionType};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::error::Error;

const FUNCTION_ERC20_NAME: &str = "0x06fdde03";
const FUNCTION_ERC20_SYMBOL: &str = "0x95d89b41";
const FUNCTION_ERC20_DECIMALS: &str = "0x313ce567";
const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
const FUNCTION_ERC20_APPROVE: &str = "0x095ea7b3";
const FUNCTION_1INCH_SWAP: &str = "0x12aa3caf";
const CONTRACT_1INCH: &str = "0x1111111254EEB25477B68fb85Ed929f73A960582";

const TOPIC_DEPOSIT: &str = "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c";
//const TOPIC_TRANSFER: &str = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

pub struct EthereumClient {
    chain: Chain,
    client: HttpClient,
}

impl EthereumClient {
    pub fn new(chain: Chain, url: String) -> Self {
        let client = HttpClientBuilder::default()
            .max_response_size(256 * 1024 * 1024) // 256MB
            .build(url)
            .unwrap();

        Self { chain, client }
    }

    async fn eth_call<T: DeserializeOwned>(&self, contract: &str, data: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let res = self
            .client
            .request(
                "eth_call",
                vec![
                    json!({
                        "to": contract,
                        "data": data,
                    }),
                    json!("latest"),
                ],
            )
            .await?;
        Ok(res)
    }

    async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .request("eth_getBlockByNumber", vec![json!(format!("0x{:x}", block_number)), json!(true)])
            .await?)
    }

    async fn get_block_reciepts(&self, block_number: i64) -> Result<Vec<TransactionReciept>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .request("eth_getBlockReceipts", vec![json!(format!("0x{:x}", block_number))])
            .await?)
    }

    fn map_transaction(&self, transaction: Transaction, transaction_reciept: &TransactionReciept, timestamp: BigUint) -> Option<primitives::Transaction> {
        let state = if transaction_reciept.status == "0x1" {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let value = transaction.value.to_string();
        let nonce = transaction.clone().nonce;
        let block_number = transaction.clone().block_number;
        let fee = transaction_reciept.get_fee().to_string();
        let from = ethereum_address_checksum(&transaction.from.clone()).ok()?;
        let to = ethereum_address_checksum(&transaction.to.clone().unwrap_or_default()).ok()?;
        let created_at = DateTime::from_timestamp(timestamp.try_into().ok()?, 0)?;

        // system transfer
        if transaction.input == "0x" {
            let transaction = primitives::Transaction::new(
                transaction.hash.clone(),
                self.chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                self.chain.as_asset_id(),
                value,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }
        // ERC20 transfer. Only add confirmed
        let input_prefix = transaction.input.chars().take(10).collect::<String>();
        if (input_prefix.starts_with(FUNCTION_ERC20_TRANSFER) || input_prefix.starts_with(FUNCTION_ERC20_APPROVE)) && state == TransactionState::Confirmed {
            let transaction_type = match input_prefix.as_str() {
                FUNCTION_ERC20_TRANSFER => TransactionType::Transfer,
                FUNCTION_ERC20_APPROVE => TransactionType::TokenApproval,
                _ => TransactionType::Transfer,
            };
            let asset_id = AssetId {
                chain: self.chain,
                token_id: to.clone().into(),
            };
            let value: String = transaction.input.chars().skip(74).take(64).collect();
            let to_address: String = transaction.input.chars().skip(34).take(40).collect::<String>();
            let to_address = ethereum_address_checksum(&to_address).ok()?;
            let value = BigUint::from_str_radix(value.as_str(), 16).unwrap_or_default();

            let transaction = primitives::Transaction::new(
                transaction.hash.clone(),
                asset_id,
                from,
                to_address.clone(),
                None,
                transaction_type,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                self.chain.as_asset_id(),
                value.to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        if input_prefix.starts_with(FUNCTION_1INCH_SWAP) && to == CONTRACT_1INCH && transaction_reciept.logs.len() <= 9 {
            let first_log = transaction_reciept.logs.first()?;
            let last_log = transaction_reciept.logs.last()?;
            let first_log_value = BigUint::from_str_radix(&first_log.clone().data[2..], 16).ok()?.to_string();
            let last_log_value = BigUint::from_str_radix(&last_log.clone().data[2..], 16).ok()?.to_string();

            let values: (String, String) = if first_log.topics[0] == TOPIC_DEPOSIT {
                (value, last_log_value.clone())
            } else {
                (first_log_value.clone(), last_log_value.clone())
            };
            let from_value = values.0.clone();
            let to_value = values.1.clone();

            let assets = if first_log.topics[0] == TOPIC_DEPOSIT {
                (
                    self.chain.as_asset_id(),
                    AssetId {
                        chain: self.chain,
                        token_id: ethereum_address_checksum(&last_log.address).ok(),
                    },
                )
            } else {
                (
                    AssetId {
                        chain: self.chain,
                        token_id: ethereum_address_checksum(&first_log.address).ok(),
                    },
                    self.chain.as_asset_id(),
                )
            };

            let swap = TransactionSwapMetadata {
                from_asset: assets.0.clone(),
                from_value: from_value.clone(),
                to_asset: assets.1.clone(),
                to_value: to_value.clone(),
                provider: None,
            };
            let asset_id = assets.clone().0;

            let transaction = primitives::Transaction::new(
                transaction.hash.clone(),
                asset_id,
                from.clone(),
                from.clone(),
                to.to_string().into(),
                TransactionType::Swap,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                self.chain.as_asset_id(),
                from_value.clone().to_string(),
                None,
                serde_json::to_value(swap).ok(),
                created_at,
            );
            return Some(transaction);
        }

        None
    }
}

#[async_trait]
impl ChainBlockProvider for EthereumClient {
    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block: String = self.client.request("eth_blockNumber", rpc_params![]).await?;
        Ok(i64::from_str_radix(&block[2..], 16)?)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?.clone();
        let transactions_reciepts = self.get_block_reciepts(block_number).await?.clone();
        let transactions = block.transactions;

        let transactions = transactions
            .into_iter()
            .zip(transactions_reciepts.iter())
            .filter_map(|(transaction, receipt)| self.map_transaction(transaction, receipt, block.timestamp.clone()))
            .collect::<Vec<primitives::Transaction>>();

        return Ok(transactions);
    }
}

#[async_trait]
impl ChainTokenDataProvider for EthereumClient {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let name: String = self.eth_call(token_id.as_str(), FUNCTION_ERC20_NAME).await?;
        let symbol: String = self.eth_call(token_id.as_str(), FUNCTION_ERC20_SYMBOL).await?;
        let decimals: String = self.eth_call(token_id.as_str(), FUNCTION_ERC20_DECIMALS).await?;

        let name: String = erc20::nameCall::abi_decode_returns(&Vec::from_hex(name)?).unwrap();
        let symbol: String = erc20::symbolCall::abi_decode_returns(&Vec::from_hex(symbol)?).unwrap();
        let decimals: u8 = erc20::decimalsCall::abi_decode_returns(&Vec::from_hex(decimals)?).unwrap();

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            name,
            symbol,
            decimals as i32,
            self.get_chain().default_asset_type().unwrap(),
        ))
    }
}
