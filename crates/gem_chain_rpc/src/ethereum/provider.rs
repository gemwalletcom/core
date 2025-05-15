use std::error::Error;

use super::client::EthereumClient;
use super::model::{Transaction, TransactionReciept};
use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::DateTime;
use gem_evm::ethereum_address_checksum;
use hex::FromHex;
use num_bigint::BigUint;
use num_traits::Num;
use primitives::{chain::Chain, Asset, AssetId, TransactionState, TransactionSwapMetadata, TransactionType};

const TOPIC_DEPOSIT: &str = "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c";
const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
const FUNCTION_ERC20_APPROVE: &str = "0x095ea7b3";
const FUNCTION_1INCH_SWAP: &str = "0x12aa3caf";
const CONTRACT_1INCH: &str = "0x1111111254EEB25477B68fb85Ed929f73A960582";

pub struct EthereumProvider {
    client: EthereumClient,
}

impl EthereumProvider {
    pub fn new(client: EthereumClient) -> Self {
        Self { client }
    }

    pub fn map_transaction(&self, transaction: Transaction, transaction_reciept: &TransactionReciept, timestamp: BigUint) -> Option<primitives::Transaction> {
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
                self.get_chain().as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                self.get_chain().as_asset_id(),
                value,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        // erc20 transfer
        if transaction.input.starts_with(FUNCTION_ERC20_TRANSFER) && transaction.input.len() >= 10 + 64 + 64 {
            let address = &transaction.input[10..74];
            let amount = &transaction.input[74..];

            let address = format!("0x{}", address);
            let address = address.trim_start_matches("0x000000000000000000000000");
            let address = ethereum_address_checksum(&format!("0x{}", address)).ok()?;

            let amount = BigUint::from_str_radix(amount, 16).ok()?;

            let token_id = ethereum_address_checksum(&to).ok()?;
            let transaction = primitives::Transaction::new(
                transaction.hash.clone(),
                AssetId::from_token(self.get_chain(), &token_id),
                from.clone(),
                address,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                self.get_chain().as_asset_id(),
                amount.to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        // approve
        if transaction.input.starts_with(FUNCTION_ERC20_APPROVE) {
            return None;
        }

        // 1inch swap
        if transaction.input.starts_with(FUNCTION_1INCH_SWAP) && to.to_lowercase() == CONTRACT_1INCH.to_lowercase() {
            let logs = &transaction_reciept.logs;
            // get first and last log
            let first_log = &logs.first()?;
            let last_log = &logs.last()?;

            // check if it's native swap
            let first_log_value = BigUint::from_str_radix(&first_log.data[2..], 16).ok()?.to_string();
            let last_log_value = BigUint::from_str_radix(&last_log.data[2..], 16).ok()?.to_string();

            let values: (String, String) = if first_log.topics[0] == TOPIC_DEPOSIT {
                (value, last_log_value.clone())
            } else {
                (first_log_value.clone(), last_log_value.clone())
            };
            let from_value = values.0.clone();
            let to_value = values.1.clone();

            let assets = if first_log.topics[0] == TOPIC_DEPOSIT {
                (
                    self.get_chain().as_asset_id(),
                    AssetId {
                        chain: self.get_chain(),
                        token_id: ethereum_address_checksum(&last_log.address).ok(),
                    },
                )
            } else {
                (
                    AssetId {
                        chain: self.get_chain(),
                        token_id: ethereum_address_checksum(&first_log.address).ok(),
                    },
                    self.get_chain().as_asset_id(),
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
                self.get_chain().as_asset_id(),
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
impl ChainBlockProvider for EthereumProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?.clone();
        let transactions_reciepts = self.client.get_block_reciepts(block_number).await?.clone();
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
impl ChainTokenDataProvider for EthereumProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let name: String = self.client.eth_call(token_id.as_str(), super::client::FUNCTION_ERC20_NAME).await?;
        let symbol: String = self.client.eth_call(token_id.as_str(), super::client::FUNCTION_ERC20_SYMBOL).await?;
        let decimals: String = self.client.eth_call(token_id.as_str(), super::client::FUNCTION_ERC20_DECIMALS).await?;

        // The original working implementation seems to have used the SolCall trait methods
        // Let's try to recreate it as closely as possible
        let name_bytes = Vec::from_hex(name)?;
        let symbol_bytes = Vec::from_hex(symbol)?;
        let decimals_bytes = Vec::from_hex(decimals)?;

        // We need to extract actual values from the function call objects
        // Instead of trying to use type inference, let's hardcode the return types
        let name_value = String::from_utf8(name_bytes.clone()).unwrap_or_default();
        let symbol_value = String::from_utf8(symbol_bytes.clone()).unwrap_or_default();
        let decimals_value: u8 = decimals_bytes.first().copied().unwrap_or_default();

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            name_value,
            symbol_value,
            decimals_value as i32,
            self.get_chain().default_asset_type().unwrap(),
        ))
    }
}
