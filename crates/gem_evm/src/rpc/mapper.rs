use crate::address::ethereum_address_checksum;

use chrono::DateTime;
use num_bigint::BigUint;
use num_traits::Num;
use primitives::{chain::Chain, AssetId, TransactionState, TransactionType};

use super::model::{Transaction, TransactionReciept};

const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";
const FUNCTION_ERC20_APPROVE: &str = "0x095ea7b3";

pub struct EthereumMapper;

impl EthereumMapper {
    pub fn map_transaction(
        chain: Chain,
        transaction: Transaction,
        transaction_reciept: &TransactionReciept,
        timestamp: BigUint,
    ) -> Option<primitives::Transaction> {
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

        None
    }
}
