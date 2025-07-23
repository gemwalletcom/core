use alloy_primitives::hex;
use chrono::{DateTime, Utc};
use num_bigint::BigUint;

use crate::{
    ethereum_address_checksum,
    rpc::model::{Transaction, TransactionReciept},
};
use primitives::{AssetId, Chain, StakeType, TransactionStakeMetadata, TransactionState, TransactionType};

const STAKE_HUB_ADDRESS: &str = "0x0000000000000000000000000000000000002002";

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
        if to_address.to_lowercase() != STAKE_HUB_ADDRESS.to_lowercase() {
            return None;
        }

        let input_bytes = hex::decode(&transaction.input).ok()?;
        if input_bytes.len() < 4 {
            return None;
        }

        let method_id = &input_bytes[0..4];
        let stake_metadata = Self::try_map_staking_transaction(chain, method_id, &input_bytes, transaction)?;

        Self::make_staking_transaction(chain, transaction, transaction_reciept, &stake_metadata, created_at)
    }

    fn try_map_staking_transaction(chain: &Chain, method_id: &[u8], input_bytes: &[u8], transaction: &Transaction) -> Option<TransactionStakeMetadata> {
        match method_id {
            // delegate(address,bool) - 0x982ef0a7
            [0x98, 0x2e, 0xf0, 0xa7] => {
                if let Ok((operator_address, _delegate_vote_power)) = gem_bsc::stake_hub::decode_delegate_call(input_bytes) {
                    if transaction.value == BigUint::from(0u32) {
                        return None;
                    }

                    Some(TransactionStakeMetadata {
                        stake_type: StakeType::Stake,
                        asset_id: AssetId::from_chain(*chain),
                        validator: Some(operator_address),
                        delegator: Some(transaction.from.clone()),
                        value: transaction.value.to_string(),
                        shares: None,
                    })
                } else {
                    None
                }
            }
            // undelegate(address,uint256) - 0x4d99dd16
            [0x4d, 0x99, 0xdd, 0x16] => {
                if let Ok((operator_address, shares)) = gem_bsc::stake_hub::decode_undelegate_call(input_bytes) {
                    Some(TransactionStakeMetadata {
                        stake_type: StakeType::Unstake,
                        asset_id: AssetId::from_chain(*chain),
                        validator: Some(operator_address),
                        delegator: Some(transaction.from.clone()),
                        value: "0".to_string(), // For undelegate, the value is in shares, not native token
                        shares: Some(shares),
                    })
                } else {
                    None
                }
            }
            // redelegate(address,address,uint256,bool) - 0x59491871
            [0x59, 0x49, 0x18, 0x71] => {
                if let Ok((src_validator, dst_validator, shares, _delegate_vote_power)) = gem_bsc::stake_hub::decode_redelegate_call(input_bytes) {
                    Some(TransactionStakeMetadata {
                        stake_type: StakeType::Redelegate,
                        asset_id: AssetId::from_chain(*chain),
                        validator: Some(format!("{src_validator}:{dst_validator}")), // Store both validators
                        delegator: Some(transaction.from.clone()),
                        value: "0".to_string(), // For redelegate, the value is in shares
                        shares: Some(shares),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn make_staking_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        metadata: &TransactionStakeMetadata,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        let from_checksum = ethereum_address_checksum(&transaction.from).ok()?;
        let contract_checksum = transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());

        Some(primitives::Transaction::new(
            transaction.hash.clone(),
            metadata.asset_id.clone(),
            from_checksum.clone(),
            from_checksum,
            contract_checksum,
            TransactionType::StakeDelegate,
            TransactionState::Confirmed,
            transaction_reciept.get_fee().to_string(),
            AssetId::from_chain(*chain),
            transaction.value.to_string(),
            None,
            serde_json::to_value(metadata).ok(),
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
        let transaction = Transaction {
            hash: "0x21442c7c30a6c1d6be3b5681275adb1f1402cef066579c4f53ec4f1c8c056ab0".to_string(),
            from: "0xf1a3687303606a6fd48179ce503164cdcbabeab6".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::parse_bytes(b"1158e460913d00000", 16).unwrap(), // 1.25 BNB in hex
            gas: 280395,
            input: "0x982ef0a7000000000000000000000000d34403249b2d82aaddb14e778422c966265e5fb50000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        };

        let receipt = create_test_receipt();
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let stake_tx = result.unwrap();

        assert_eq!(stake_tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(stake_tx.from, "0xf1a3687303606a6fD48179Ce503164CDcBAbeaB6");
        assert_eq!(stake_tx.to, "0xf1a3687303606a6fD48179Ce503164CDcBAbeaB6");
        assert_eq!(stake_tx.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(stake_tx.value, "20000000000000000000");

        let metadata: TransactionStakeMetadata = serde_json::from_value(stake_tx.metadata.unwrap()).unwrap();
        assert_eq!(metadata.stake_type, StakeType::Stake);
        assert_eq!(metadata.asset_id, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(metadata.validator.unwrap(), "0xd34403249B2d82AAdDB14e778422c966265e5Fb5");
        assert_eq!(metadata.delegator.unwrap(), "0xf1a3687303606a6fd48179ce503164cdcbabeab6");
        assert_eq!(metadata.value, "20000000000000000000");
        assert!(metadata.shares.is_none());
    }

    #[test]
    fn test_map_undelegate_transaction() {
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
        let stake_tx = result.unwrap();

        assert_eq!(stake_tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(stake_tx.from, "0x9Ccd32825A39209E0d14a5d8D27e9E545B901d8F");
        assert_eq!(stake_tx.to, "0x9Ccd32825A39209E0d14a5d8D27e9E545B901d8F");
        assert_eq!(stake_tx.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(stake_tx.value, "0");

        let metadata: TransactionStakeMetadata = serde_json::from_value(stake_tx.metadata.unwrap()).unwrap();
        assert_eq!(metadata.stake_type, StakeType::Unstake);
        assert_eq!(metadata.asset_id, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(metadata.validator.unwrap(), "0x5c38FF8Ca2b16099C086bF36546e99b13D152C4c");
        assert_eq!(metadata.delegator.unwrap(), "0x9ccd32825a39209e0d14a5d8d27e9e545b901d8f");
        assert_eq!(metadata.value, "0");
        assert_eq!(metadata.shares.unwrap(), "415946947323922080");
    }

    #[test]
    fn test_map_redelegate_transaction() {
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
        let stake_tx = result.unwrap();

        assert_eq!(stake_tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(stake_tx.from, "0xB5a0A71Be7B79F2A8Bd19B3A4D54d1b85fA2d50b");
        assert_eq!(stake_tx.to, "0xB5a0A71Be7B79F2A8Bd19B3A4D54d1b85fA2d50b");
        assert_eq!(stake_tx.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(stake_tx.value, "0");

        let metadata: TransactionStakeMetadata = serde_json::from_value(stake_tx.metadata.unwrap()).unwrap();
        assert_eq!(metadata.stake_type, StakeType::Redelegate);
        assert_eq!(metadata.asset_id, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(
            metadata.validator.unwrap(),
            "0x0813D0D092b97C157A8e68A65ccdF41b956883ae:0xB58ac55EB6B10e4f7918D77C92aA1cF5bB2DEd5e"
        );
        assert_eq!(metadata.delegator.unwrap(), "0xb5a0a71be7b79f2a8bd19b3a4d54d1b85fa2d50b");
        assert_eq!(metadata.value, "0");
        assert_eq!(metadata.shares.unwrap(), "2337013854984033567");
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
