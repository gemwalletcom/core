use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use num_traits::Num;

use crate::{
    ethereum_address_checksum,
    rpc::model::{Log, Transaction, TransactionReciept},
};
use gem_bsc::stake_hub;
use primitives::{AssetId, Chain, TransactionType};

const EVENT_BSC_DELEGATED: &str = "0x24d7bda8602b916d64417f0dbfe2e2e88ec9b1157bd9f596dfdb91ba26624e04";
const EVENT_BSC_UNDELEGATED: &str = "0x3aace7340547de7b9156593a7652dc07ee900cea3fd8f82cb6c9d38b40829802";
const EVENT_BSC_REDELEGATED: &str = "0xfdac6e81913996d95abcc289e90f2d8bd235487ce6fe6f821e7d21002a1915b4";
const EVENT_BSC_CLAIMED: &str = "0xf7a40077ff7a04c7e61f6f26fb13774259ddf1b6bce9ecf26a8276cdd3992683";

pub struct StakingMapper;

impl StakingMapper {
    pub fn map_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        match chain {
            Chain::SmartChain => {
                if transaction_reciept.logs.is_empty() || transaction.to.as_ref().map(|to| to == stake_hub::STAKE_HUB_ADDRESS).is_none() {
                    return None;
                }
                Self::map_smartchain_staking_transaction(chain, transaction, transaction_reciept, created_at)
            }
            _ => None,
        }
    }

    fn map_smartchain_staking_transaction(
        chain: &Chain,
        transaction: &Transaction,
        transaction_reciept: &TransactionReciept,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        for log in &transaction_reciept.logs {
            if let Some(event_signature) = log.topics.first() {
                match event_signature.as_str() {
                    EVENT_BSC_DELEGATED => {
                        return Self::parse_delegated_event(chain, transaction, transaction_reciept, log, created_at);
                    }
                    EVENT_BSC_UNDELEGATED => {
                        return Self::parse_undelegated_event(chain, transaction, transaction_reciept, log, created_at);
                    }
                    EVENT_BSC_REDELEGATED => {
                        return Self::parse_redelegated_event(chain, transaction, transaction_reciept, log, created_at);
                    }
                    EVENT_BSC_CLAIMED => {
                        return Self::parse_claimed_event(chain, transaction, transaction_reciept, log, created_at);
                    }
                    _ => continue,
                }
            }
        }
        None
    }

    fn parse_delegated_event(
        chain: &Chain,
        transaction: &Transaction,
        reciept: &TransactionReciept,
        log: &Log,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        if log.topics.len() != 3 {
            return None;
        }

        let operator_address = ethereum_address_checksum(log.topics[1].trim_start_matches("0x000000000000000000000000")).ok()?;

        let data_without_prefix = log.data.trim_start_matches("0x");
        if data_without_prefix.len() < 128 {
            return None;
        }
        let bnb_amount_hex = &data_without_prefix[64..128];
        let bnb_amount = BigUint::from_str_radix(bnb_amount_hex, 16).ok()?;

        Self::create_staking_transaction(
            chain,
            transaction,
            reciept,
            &operator_address,
            TransactionType::StakeDelegate,
            &bnb_amount.to_string(),
            created_at,
        )
    }

    fn parse_undelegated_event(
        chain: &Chain,
        transaction: &Transaction,
        reciept: &TransactionReciept,
        log: &Log,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        if log.topics.len() != 3 {
            return None;
        }

        let operator_address = ethereum_address_checksum(log.topics[1].trim_start_matches("0x000000000000000000000000")).ok()?;
        let _delegator = ethereum_address_checksum(log.topics[2].trim_start_matches("0x000000000000000000000000")).ok()?;

        let data_without_prefix = log.data.trim_start_matches("0x");
        if data_without_prefix.len() < 128 {
            return None;
        }
        let bnb_amount_hex = &data_without_prefix[64..128];
        let bnb_amount = BigUint::from_str_radix(bnb_amount_hex, 16).ok()?;

        Self::create_staking_transaction(
            chain,
            transaction,
            reciept,
            &operator_address,
            TransactionType::StakeUndelegate,
            &bnb_amount.to_string(),
            created_at,
        )
    }

    fn parse_redelegated_event(
        chain: &Chain,
        transaction: &Transaction,
        reciept: &TransactionReciept,
        log: &Log,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        if log.topics.len() != 4 {
            return None;
        }

        let dst_validator = ethereum_address_checksum(log.topics[2].trim_start_matches("0x000000000000000000000000")).ok()?;

        let data_without_prefix = log.data.trim_start_matches("0x");
        if data_without_prefix.len() < 192 {
            return None;
        }
        let bnb_amount_hex = &data_without_prefix[128..192];
        let bnb_amount = BigUint::from_str_radix(bnb_amount_hex, 16).ok()?;

        Self::create_staking_transaction(
            chain,
            transaction,
            reciept,
            &dst_validator,
            TransactionType::StakeRedelegate,
            &bnb_amount.to_string(),
            created_at,
        )
    }

    fn parse_claimed_event(
        chain: &Chain,
        transaction: &Transaction,
        reciept: &TransactionReciept,
        log: &Log,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        if log.topics.len() != 3 {
            return None;
        }

        let operator_address = ethereum_address_checksum(log.topics[1].trim_start_matches("0x000000000000000000000000")).ok()?;
        let _delegator = ethereum_address_checksum(log.topics[2].trim_start_matches("0x000000000000000000000000")).ok()?;

        let data_without_prefix = log.data.trim_start_matches("0x");
        if data_without_prefix.len() < 64 {
            return None;
        }
        let bnb_amount_hex = &data_without_prefix[0..64];
        let bnb_amount = BigUint::from_str_radix(bnb_amount_hex, 16).ok()?;

        Self::create_staking_transaction(
            chain,
            transaction,
            reciept,
            &operator_address,
            TransactionType::StakeRewards,
            &bnb_amount.to_string(),
            created_at,
        )
    }

    fn create_staking_transaction(
        chain: &Chain,
        transaction: &Transaction,
        reciept: &TransactionReciept,
        validator_address: &str,
        transaction_type: TransactionType,
        value: &str,
        created_at: DateTime<Utc>,
    ) -> Option<primitives::Transaction> {
        let from = ethereum_address_checksum(&transaction.from).ok()?;
        let to = ethereum_address_checksum(validator_address).ok()?;
        let contract = transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());
        let state = reciept.get_state();

        Some(primitives::Transaction::new(
            transaction.hash.clone(),
            AssetId::from_chain(*chain),
            from,
            to,
            contract,
            transaction_type,
            state,
            reciept.get_fee().to_string(),
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
    use crate::rpc::model::{Log, Transaction, TransactionReciept};
    use num_bigint::BigUint;

    fn create_test_receipt_with_log(log: Log) -> TransactionReciept {
        TransactionReciept {
            gas_used: BigUint::from(100000u32),
            effective_gas_price: BigUint::from(20000000000u64),
            l1_fee: None,
            logs: vec![log],
            status: "0x1".to_string(),
            block_number: "0x1234".to_string(),
        }
    }

    #[test]
    fn test_map_delegate_transaction() {
        // https://bscscan.com/tx/0xd85c4496230adf8a7c0fc1e98713127fb31a0f8f72874acea443e2f615f3c1b6
        let transaction = Transaction {
            hash: "0xd85c4496230adf8a7c0fc1e98713127fb31a0f8f72874acea443e2f615f3c1b6".to_string(),
            from: "0x51ed60604637989d19d29e43c5d94b098a0d1af7".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(1000000000000000000u64),
            gas: 280395,
            input: "0x".to_string(),
        };

        let log = Log {
            address: "0x0000000000000000000000000000000000002002".to_string(),
            topics: vec![
                EVENT_BSC_DELEGATED.to_string(),
                "0x000000000000000000000000d34403249B2d82AAdDB14e778422c966265e5Fb5".to_string(),
                "0x00000000000000000000000051eD60604637989d19D29e43c5D94B098A0d1Af7".to_string(),
            ],
            data: "0x00000000000000000000000000000000000000000000000d5cc0065cf2d900aa0000000000000000000000000000000000000000000000000de0b6b3a7640000"
                .to_string(),
        };

        let receipt = create_test_receipt_with_log(log);
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(transaction.from, "0x51eD60604637989d19D29e43c5D94B098A0d1Af7");
        assert_eq!(transaction.to, "0xd34403249B2d82AAdDB14e778422c966265e5Fb5");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "1000000000000000000");
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_map_undelegate_transaction() {
        // https://bscscan.com/tx/0x564b45165bf777355c6e7de2dbd5b25f7cef5862385eb7cd67795c47f4358620#eventlog
        let transaction = Transaction {
            hash: "0x564b45165bf777355c6e7de2dbd5b25f7cef5862385eb7cd67795c47f4358620".to_string(),
            from: "0xa103B70852B1fE3eF3a0B60B818279F9D0D337d9".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(0u32),
            gas: 384404,
            input: "0x".to_string(),
        };

        let log = Log {
            address: "0x0000000000000000000000000000000000002002".to_string(),
            topics: vec![
                EVENT_BSC_UNDELEGATED.to_string(),
                "0x0000000000000000000000005c38FF8Ca2b16099C086bF36546e99b13D152C4c".to_string(),
                "0x000000000000000000000000a103B70852B1fE3eF3a0B60B818279F9D0D337d9".to_string(),
            ],
            data: "0x0000000000000000000000000000000000000000000000000e539ee6df39e04c0000000000000000000000000000000000000000000000000e83bec8de346b99"
                .to_string(),
        };

        let receipt = create_test_receipt_with_log(log);
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeUndelegate);
        assert_eq!(transaction.from, "0xa103B70852B1fE3eF3a0B60B818279F9D0D337d9");
        assert_eq!(transaction.to, "0x5c38FF8Ca2b16099C086bF36546e99b13D152C4c");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "1045889308410801049");
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_map_redelegate_transaction() {
        // https://bscscan.com/tx/0xc31c1ff67a9b6784d5eb2aafe51fb8d93c64034514ab7423a0d12aa8ced3ee9c#eventlog
        let transaction = Transaction {
            hash: "0xc31c1ff67a9b6784d5eb2aafe51fb8d93c64034514ab7423a0d12aa8ced3ee9c".to_string(),
            from: "0xb5a0a71be7b79f2a8bd19b3a4d54d1b85fa2d50b".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(0u32),
            gas: 485626,
            input: "0x".to_string(),
        };

        let log = Log {
            address: "0x0000000000000000000000000000000000002002".to_string(),
            topics: vec![
                EVENT_BSC_REDELEGATED.to_string(),
                "0x0000000000000000000000000813D0D092b97C157A8e68A65ccdF41b956883ae".to_string(),
                "0x000000000000000000000000B58ac55EB6B10e4f7918D77C92aA1cF5bB2DEd5e".to_string(),
                "0x000000000000000000000000B5a0A71Be7B79F2A8Bd19B3A4D54d1b85fA2d50b".to_string(),
            ],
            data: "0x000000000000000000000000000000000000000000000000206ebdb8157d551f0000000000000000000000000000000000000000000000002068edb30143ec5300000000000000000000000000000000000000000000000020e60fe483aabb11".to_string(),
        };

        let receipt = create_test_receipt_with_log(log);
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeRedelegate);
        assert_eq!(transaction.from, "0xB5a0A71Be7B79F2A8Bd19B3A4D54d1b85fA2d50b");
        assert_eq!(transaction.to, "0xB58ac55EB6B10e4f7918D77C92aA1cF5bB2DEd5e");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "2370599727993109265");
        assert!(transaction.metadata.is_none());
    }

    #[test]
    fn test_map_claim_transaction() {
        // https://bscscan.com/tx/0xdf26bfaf989ac4f17b425fb36cc14b64332d0390f67e95a70fca875860fc14d9#eventlog
        let transaction = Transaction {
            hash: "0xdf26bfaf989ac4f17b425fb36cc14b64332d0390f67e95a70fca875860fc14d9".to_string(),
            from: "0x47B47f2586089F68Ec17384a437F96800f499274".to_string(),
            to: Some("0x0000000000000000000000000000000000002002".to_string()),
            value: BigUint::from(0u32),
            gas: 150000,
            input: "0x".to_string(),
        };

        let log = Log {
            address: "0x0000000000000000000000000000000000002002".to_string(),
            topics: vec![
                EVENT_BSC_CLAIMED.to_string(),
                "0x000000000000000000000000B12e8137eF499a1d81552DB11664a9E617fd350A".to_string(),
                "0x00000000000000000000000047B47f2586089F68Ec17384a437F96800f499274".to_string(),
            ],
            data: "0x0000000000000000000000000000000000000000000000003786b5ea2b989d0d".to_string(),
        };

        let receipt = create_test_receipt_with_log(log);
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_some());
        let transaction = result.unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::StakeRewards);
        assert_eq!(transaction.from, "0x47B47f2586089F68Ec17384a437F96800f499274");
        assert_eq!(transaction.to, "0xB12e8137eF499a1d81552DB11664a9E617fd350A");
        assert_eq!(transaction.contract.unwrap(), "0x0000000000000000000000000000000000002002");
        assert_eq!(transaction.value, "4001085336323661069");
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
            input: "0x".to_string(),
        };

        let log = Log {
            address: "0x0000000000000000000000000000000000002002".to_string(),
            topics: vec![
                EVENT_BSC_DELEGATED.to_string(),
                "0x000000000000000000000000d34403249B2d82AAdDB14e778422c966265e5Fb5".to_string(),
                "0x0000000000000000000000001234567890123456789012345678901234567890".to_string(),
            ],
            data: "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de0b6b3a7640000"
                .to_string(),
        };

        let receipt = create_test_receipt_with_log(log);
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

        let log = Log {
            address: "0x1234567890123456789012345678901234567890".to_string(),
            topics: vec![
                EVENT_BSC_DELEGATED.to_string(),
                "0x000000000000000000000000d34403249B2d82AAdDB14e778422c966265e5Fb5".to_string(),
                "0x0000000000000000000000001234567890123456789012345678901234567890".to_string(),
            ],
            data: "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000de0b6b3a7640000"
                .to_string(),
        };

        let receipt = create_test_receipt_with_log(log);
        let result = StakingMapper::map_transaction(&Chain::SmartChain, &transaction, &receipt, DateTime::default());

        assert!(result.is_none());
    }
}
