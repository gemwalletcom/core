use chrono::DateTime;
use num_bigint::BigUint;
use num_traits::Num;

use super::swap_mapper::SwapMapper;
use crate::{
    address::ethereum_address_checksum,
    rpc::model::{Block, Transaction, TransactionReciept},
};
use primitives::{chain::Chain, AssetId, TransactionState, TransactionType};

pub const INPUT_0X: &str = "0x";
pub const FUNCTION_ERC20_TRANSFER: &str = "0xa9059cbb";

pub struct EthereumMapper;

impl EthereumMapper {
    pub fn map_transactions(chain: Chain, block: Block, transactions_reciepts: Vec<TransactionReciept>) -> Vec<primitives::Transaction> {
        let timestamp = block.timestamp.clone();
        block
            .transactions
            .into_iter()
            .zip(transactions_reciepts.iter())
            .filter_map(|(transaction, receipt)| {
                EthereumMapper::map_transaction(chain, &transaction, receipt, timestamp.clone())
            })
            .collect()
    }

    pub fn map_transaction(
        chain: Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        timestamp: BigUint,
    ) -> Option<primitives::Transaction> {
        let state = if transaction_reciept.status == "0x1" {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };
        let value = transaction.value.to_string();
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
                fee.to_string(),
                chain.as_asset_id(),
                amount.to_string(),
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }

        // Try to decode Uniswap V3 or V4 transaction
        if transaction.to.is_some() && transaction.input.len() >= 8 {
            if let Some(tx) = SwapMapper::map_uniswap_transaction(&chain, transaction, transaction_reciept, created_at) {
                return Some(tx);
            }
        }

        // Check for smart contract call using intrinsic transaction properties
        let is_smart_contract_call = transaction.to.is_some() 
            && !transaction.input.is_empty() 
            && transaction.input != INPUT_0X
            && Self::has_smart_contract_indicators(transaction, transaction_reciept);

        if is_smart_contract_call {
            let transaction = primitives::Transaction::new(
                transaction.hash.clone(),
                chain.as_asset_id(),
                from,
                to,
                None,
                TransactionType::SmartContractCall,
                state,
                fee.to_string(),
                chain.as_asset_id(),
                value,
                None,
                None,
                created_at,
            );
            return Some(transaction);
        }
        None
    }

    fn has_smart_contract_indicators(transaction: &Transaction, transaction_reciept: &TransactionReciept) -> bool {
        // Smart contract calls typically have:
        // 1. Gas limit > 21,000 (simple transfers use exactly 21,000)
        // 2. Receipt has logs (contract execution emits events)
        let gas_limit = Self::parse_hex_to_u64(&transaction.gas).unwrap_or(0);
        let has_logs = !transaction_reciept.logs.is_empty();
        
        gas_limit > 21_000 || has_logs
    }

    fn parse_hex_to_u64(hex_str: &str) -> Option<u64> {
        if let Some(stripped) = hex_str.strip_prefix("0x") {
            u64::from_str_radix(stripped, 16).ok()
        } else {
            hex_str.parse().ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{Transaction, TransactionReciept};
    use num_bigint::BigUint;
    use primitives::{Chain, TransactionType};

    #[test]
    fn test_map_smart_contract_call() {
        let contract_call_tx_json: serde_json::Value = serde_json::from_str(include_str!("test/contract_call_tx.json")).unwrap();
        let contract_call_tx: Transaction = serde_json::from_value(contract_call_tx_json["result"].clone()).unwrap();

        let contract_call_receipt_json: serde_json::Value = serde_json::from_str(include_str!("test/contract_call_tx_receipt.json")).unwrap();
        let contract_call_receipt: TransactionReciept = serde_json::from_value(contract_call_receipt_json["result"].clone()).unwrap();

        let timestamp = BigUint::from(1735671600u64);

        // Test smart contract call detection using intrinsic properties
        let result = EthereumMapper::map_transaction(Chain::Ethereum, &contract_call_tx, &contract_call_receipt, timestamp);

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.hash, "0x876707912c2d625723aa14bf268d83ede36c2657c70da500628e40e6b51577c9");
        assert_eq!(transaction.from, "0x39ab5f6f1269590225EdAF9ad4c5967B09243747");
        assert_eq!(transaction.to, "0xB907Dcc926b5991A149d04Cb7C0a4a25dC2D8f9a");
    }

    #[test]
    fn test_has_smart_contract_indicators() {
        let contract_call_tx_json: serde_json::Value = serde_json::from_str(include_str!("test/contract_call_tx.json")).unwrap();
        let contract_call_tx: Transaction = serde_json::from_value(contract_call_tx_json["result"].clone()).unwrap();
        
        let contract_call_receipt_json: serde_json::Value = serde_json::from_str(include_str!("test/contract_call_tx_receipt.json")).unwrap();
        let contract_call_receipt: TransactionReciept = serde_json::from_value(contract_call_receipt_json["result"].clone()).unwrap();

        // Should detect smart contract call (has logs and gas > 21000)
        assert!(EthereumMapper::has_smart_contract_indicators(&contract_call_tx, &contract_call_receipt));
        
        // Test gas parsing
        assert_eq!(EthereumMapper::parse_hex_to_u64("0x61a80"), Some(400000)); // > 21000
        assert_eq!(EthereumMapper::parse_hex_to_u64("0x5208"), Some(21000)); // exactly 21000
    }
}
