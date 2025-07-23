use alloy_primitives::hex;
use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{
    ethereum_address_checksum,
    rpc::model::{Transaction, TransactionReciept},
};
use gem_bsc::stake_hub;
use primitives::{AssetId, Chain, TransactionState, TransactionType};

// BSC staking method signatures
const FUNCTION_BSC_DELEGATE: &str = "0x982ef0a7"; // delegate(address,bool)
const FUNCTION_BSC_UNDELEGATE: &str = "0x4d99dd16"; // undelegate(address,uint256)  
const FUNCTION_BSC_REDELEGATE: &str = "0x59491871"; // redelegate(address,address,uint256,bool)

pub struct StakingMapper;

impl StakingMapper {
    pub fn map_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        // Only handle SmartChain (BSC) staking transactions for now
        if *chain != Chain::SmartChain {
            return None;
        }

        // Check if transaction is to the StakeHub contract
        let to_address = transaction.to.as_ref()?;
        if to_address != stake_hub::STAKE_HUB_ADDRESS {
            return None;
        }

        let input_bytes = hex::decode(&transaction.input).ok()?;
        if input_bytes.len() < 4 {
            return None;
        }

        let method_id = &input_bytes[0..4];
        Self::try_map_staking_transaction(chain, method_id, &input_bytes, transaction, transaction_reciept, created_at)
    }

    fn try_map_staking_transaction(
        chain: &Chain,
        method_id: &[u8],
        input_bytes: &[u8],
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        let method_hex = hex::encode(method_id);
        let method_signature = format!("0x{method_hex}");
        
        match method_signature.as_str() {
            FUNCTION_BSC_DELEGATE => {
                if let Ok((operator_address, _delegate_vote_power)) = stake_hub::decode_delegate_call(input_bytes) {
                    if transaction.value == BigUint::from(0u32) {
                        return None;
                    }

                    Self::create_staking_transaction(
                        chain,
                        transaction,
                        transaction_reciept,
                        &operator_address,
                        TransactionType::StakeDelegate,
                        &transaction.value.to_string(),
                        created_at,
                    )
                } else {
                    None
                }
            }
            FUNCTION_BSC_UNDELEGATE => {
                if let Ok((operator_address, _shares)) = stake_hub::decode_undelegate_call(input_bytes) {
                    Self::create_staking_transaction(
                        chain,
                        transaction,
                        transaction_reciept,
                        &operator_address,
                        TransactionType::StakeUndelegate,
                        "0", // For undelegate, the value is in shares, not native token
                        created_at,
                    )
                } else {
                    None
                }
            }
            FUNCTION_BSC_REDELEGATE => {
                if let Ok((_src_validator, dst_validator, _shares, _delegate_vote_power)) = stake_hub::decode_redelegate_call(input_bytes) {
                    Self::create_staking_transaction(
                        chain,
                        transaction,
                        transaction_reciept,
                        &dst_validator,
                        TransactionType::StakeRedelegate,
                        "0", // For redelegate, the value is in shares
                        created_at,
                    )
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn create_staking_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        validator_address: &str,
        transaction_type: TransactionType,
        value: &str,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        let from_checksum = ethereum_address_checksum(&transaction.from).ok()?;
        let to_checksum = ethereum_address_checksum(validator_address).ok()?;
        let contract_checksum = transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());
        let state = if transaction_reciept.status == "0x1" {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };

        Some(primitives::Transaction::new(
            transaction.hash.clone(),
            AssetId::from_chain(*chain),
            from_checksum,
            to_checksum,
            contract_checksum,
            transaction_type,
            state,
            transaction_reciept.get_fee().to_string(),
            AssetId::from_chain(*chain),
            value.to_string(),
            None,
            None,
            created_at,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{Transaction, TransactionReciept};

    fn create_test_receipt() -> TransactionReciept {
        TransactionReciept {
            gas_used: BigUint::from(21000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_number: "0x1234".to_string(),
        }
    }

    #[test]
    fn test_map_delegate_transaction() {
        // https://bscscan.com/tx/0x21442c7c30a6c1d6be3b5681275adb1f1402cef066579c4f53ec4f1c8c056ab0
        let transaction = Transaction {
            hash: "0x21442c7c30a6c1d6be3b5681275adb1f1402cef066579c4f53ec4f1c8c056ab0".to_string(),
            from: "0xf1a3687303606a6fd48179ce503164cdcbabeab6".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::parse_bytes(b"1158e460913d00000", 16).unwrap(), // 1.25 BNB in wei (hex)
            gas: 280395,
            input: "0x982ef0a7000000000000000000000000d34403249b2d82aaddb14e778422c966265e5fb50000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };

        let receipt = create_test_receipt();
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(transaction.from, "0xf1a3687303606a6fD48179Ce503164CDcBAbeaB6");
        assert_eq!(transaction.to, "0xd34403249B2d82AAdDB14e778422c966265e5Fb5");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "20000000000000000000");
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_map_undelegate_transaction() {
        // https://bscscan.com/tx/0x7afc2d0a7c5a5fdc18cd61d4e699138e75bf338b972554f78b0b761f63727b39
        let transaction = Transaction {
            hash: "0x7afc2d0a7c5a5fdc18cd61d4e699138e75bf338b972554f78b0b761f63727b39".to_string(),
            from: "0x9ccd32825a39209e0d14a5d8d27e9e545b901d8f".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(0u32),
            gas: 384404,
            input: "0x4d99dd160000000000000000000000005c38ff8ca2b16099c086bf36546e99b13d152c4c00000000000000000000000000000000000000000000000005c5bd8b78a686a0"
                .to_string(),
        };

        let receipt = create_test_receipt();
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(transaction.from, "0x9Ccd32825A39209E0d14a5d8D27e9E545B901d8F");
        assert_eq!(transaction.to, "0x5c38FF8Ca2b16099C086bF36546e99b13D152C4c");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "0");
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_map_redelegate_transaction() {
        // https://bscscan.com/tx/0xc31c1ff67a9b6784d5eb2aafe51fb8d93c64034514ab7423a0d12aa8ced3ee9c
        let transaction = Transaction {
            hash: "0xc31c1ff67a9b6784d5eb2aafe51fb8d93c64034514ab7423a0d12aa8ced3ee9c".to_string(),
            from: "0xb5a0a71be7b79f2a8bd19b3a4d54d1b85fa2d50b".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(0u32),
            gas: 485626,
            input: "0x594918710000000000000000000000000813d0d092b97c157a8e68a65ccdf41b956883ae000000000000000000000000b58ac55eb6b10e4f7918d77c92aa1cf5bb2ded5e000000000000000000000000000000000000000000000000206ebdb8157d551f0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        };

        let receipt = create_test_receipt();
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeRedelegate);
        assert_eq!(transaction.from, "0xB5a0A71Be7B79F2A8Bd19B3A4D54d1b85fA2d50b");
        assert_eq!(transaction.to, "0xB58ac55EB6B10e4f7918D77C92aA1cF5bB2DEd5e");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "0");
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_non_bsc_chain_returns_none() {
        let transaction = Transaction {
            hash: "0x1234".to_string(),
            from: "0x1234".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(0u32),
            gas: 21000,
            input: "0x982ef0a7000000000000000000000000d34403249b2d82aaddb14e778422c966265e5fb50000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };

        let receipt = create_test_receipt();
        let result = StakingMapper::map_transaction(&Chain::Ethereum, &transaction, &receipt, DateTime::default());

        assert!(result.is_none());
    }

    #[test]
    fn test_non_stakehub_contract_returns_none() {
        let transaction = Transaction {
            hash: "0x1234".to_string(),
            from: "0x1234".to_string(),
            to: Some("0x1234567890123456789012345678901234567890".to_string()),
            value: BigUint::from(0u32),
            gas: 21000,
            input: "0x982ef0a7000000000000000000000000d34403249b2d82aaddb14e778422c966265e5fb50000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };

        let receipt = create_test_receipt();
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_none());
    }
}
