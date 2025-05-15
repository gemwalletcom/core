use chrono::DateTime;
use gem_evm::ethereum_address_checksum;
use num_bigint::BigUint;
use num_traits::Num;
use primitives::{chain::Chain, AssetId, TransactionState, TransactionSwapMetadata, TransactionType};

use super::model::{Transaction, TransactionReciept};

const TOPIC_DEPOSIT: &str = "0xe1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c";
const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
const FUNCTION_ERC20_APPROVE: &str = "0x095ea7b3";
const FUNCTION_1INCH_SWAP: &str = "0x12aa3caf";
const CONTRACT_1INCH: &str = "0x1111111254EEB25477B68fb85Ed929f73A960582";

pub struct EthereumMapper;

impl EthereumMapper {
    pub fn map_transaction(chain: Chain, transaction: Transaction, transaction_reciept: &TransactionReciept, timestamp: BigUint) -> Option<primitives::Transaction> {
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
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                chain.as_asset_id(),
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
                AssetId::from_token(chain, &token_id),
                from.clone(),
                address,
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                nonce.to_string(),
                fee.to_string(),
                chain.as_asset_id(),
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

            // extract values
            let first_log_value = first_log.data.clone().strip_prefix("0x")
                .map(|hex| BigUint::from_str_radix(hex, 16).ok())
                .unwrap_or_default()
                .unwrap_or_default();
                
            let last_log_value = last_log.data.clone().strip_prefix("0x")
                .map(|hex| BigUint::from_str_radix(hex, 16).ok())
                .unwrap_or_default()
                .unwrap_or_default();

            let (from_value, to_value) = if first_log.topics[0] == TOPIC_DEPOSIT {
                (value.clone(), last_log_value.to_string())
            } else {
                (first_log_value.to_string(), value.clone())
            };

            let assets = if first_log.topics[0] == TOPIC_DEPOSIT {
                (
                    chain.as_asset_id(),
                    AssetId {
                        chain,
                        token_id: ethereum_address_checksum(&last_log.address).ok(),
                    },
                )
            } else {
                (
                    AssetId {
                        chain,
                        token_id: ethereum_address_checksum(&first_log.address).ok(),
                    },
                    chain.as_asset_id(),
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
                chain.as_asset_id(),
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
