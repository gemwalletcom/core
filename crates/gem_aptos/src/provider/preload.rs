use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use super::preload_mapper::map_transaction_preload;
use crate::provider::payload_builder::{build_stake_payload_data, build_unstake_payload_data, build_withdraw_payload_data};
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for AptosClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&input.sender_address).await?;
        map_transaction_preload(&account)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let gas_limit = self.calculate_gas_limit(&input).await?;
        let fee = TransactionFee::calculate(gas_limit, &input.gas_price);

        let data = match &input.input_type {
            TransactionInputType::Stake(_, stake_type) => match stake_type {
                StakeType::Stake(validator) => Some(build_stake_payload_data(&validator.id, &input.value)),
                StakeType::Unstake(delegation) => Some(build_unstake_payload_data(&delegation.validator.id, &input.value)),
                StakeType::Withdraw(delegation) => Some(build_withdraw_payload_data(&delegation.validator.id, &input.value)),
                StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => None,
            },
            _ => None,
        };

        let sequence = input.metadata.get_sequence()?;
        let metadata = TransactionLoadMetadata::Aptos { sequence, data };

        Ok(TransactionLoadData { fee, metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_fee = self.get_gas_price().await?;

        Ok(vec![
            FeeRate::new(FeePriority::Slow, GasPriceType::regular(gas_fee.deprioritized_gas_estimate)),
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(gas_fee.gas_estimate)),
            FeeRate::new(FeePriority::Fast, GasPriceType::regular(gas_fee.prioritized_gas_estimate)),
        ])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::KNOWN_VALIDATOR_POOL;
    use crate::provider::testkit::{TEST_ADDRESS_STAKING, create_aptos_test_client};
    use num_bigint::BigInt;
    use primitives::{Asset, Chain, DelegationValidator};
    use serde_json::Value;

    #[tokio::test]
    async fn test_aptos_get_transaction_load_stake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let metadata = client
            .get_transaction_preload(TransactionPreloadInput {
                input_type: TransactionInputType::Stake(
                    Asset::from_chain(Chain::Aptos),
                    StakeType::Stake(DelegationValidator::stake(Chain::Aptos, KNOWN_VALIDATOR_POOL.to_string(), String::new(), true, 0.0, 0.0)),
                ),
                sender_address: TEST_ADDRESS_STAKING.to_string(),
                destination_address: KNOWN_VALIDATOR_POOL.to_string(),
            })
            .await?;

        let load = client
            .get_transaction_load(TransactionLoadInput {
                input_type: TransactionInputType::Stake(
                    Asset::from_chain(Chain::Aptos),
                    StakeType::Stake(DelegationValidator::stake(Chain::Aptos, KNOWN_VALIDATOR_POOL.to_string(), String::new(), true, 0.0, 0.0)),
                ),
                sender_address: TEST_ADDRESS_STAKING.to_string(),
                destination_address: KNOWN_VALIDATOR_POOL.to_string(),
                value: "1100000000".to_string(),
                gas_price: GasPriceType::regular(BigInt::from(100u64)),
                memo: None,
                is_max_value: false,
                metadata,
            })
            .await?;

        let TransactionLoadMetadata::Aptos { data: Some(data), .. } = load.metadata else {
            panic!("Expected Aptos transaction load metadata with payload");
        };

        let payload: Value = serde_json::from_str(&data).unwrap();
        assert_eq!(payload["function"], "0x1::delegation_pool::add_stake");
        assert!(load.fee.gas_limit > BigInt::from(0u32));

        Ok(())
    }
}
