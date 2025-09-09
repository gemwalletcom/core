use crate::fee_calculator::{get_fee_history_blocks, get_reward_percentiles};
#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionLoad;
#[cfg(feature = "rpc")]
use num_bigint::BigInt;
#[cfg(feature = "rpc")]
use primitives::{FeeRate, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};
#[cfg(feature = "rpc")]
use serde_serializers::bigint::bigint_from_hex_str;
use std::collections::HashMap;
use std::error::Error;

#[cfg(feature = "rpc")]
use super::preload_optimism::OptimismGasOracle;
use crate::provider::preload_mapper::{
    bigint_to_hex_string, bytes_to_hex_string, calculate_gas_limit_with_increase, get_extra_fee_gas_limit, get_transaction_data, get_transaction_to,
    get_transaction_value, map_transaction_fee_rates, map_transaction_preload,
};
use crate::rpc::client::EthereumClient;
use gem_client::Client;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for EthereumClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let nonce = self.get_transaction_count(&input.sender_address).await?;
        let chain_id = self.chain.to_chain().network_id().to_string();

        map_transaction_preload(nonce, chain_id)
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let fee_history = self
            .get_fee_history(get_fee_history_blocks(self.chain), get_reward_percentiles().to_vec())
            .await?;

        map_transaction_fee_rates(self.chain, &fee_history)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        self.map_transaction_load(input).await
    }
}

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn map_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let data = get_transaction_data(self.chain, &input)?;
        let to = get_transaction_to(self.chain, &input)?;
        let value = get_transaction_value(self.chain, &input)?;

        let gas_estimate = {
            let estimate = self
                .estimate_gas(
                    &input.sender_address,
                    &to,
                    Some(&bigint_to_hex_string(&value)),
                    Some(&bytes_to_hex_string(&data)),
                )
                .await?;
            bigint_from_hex_str(&estimate)?
        };
        let gas_limit = calculate_gas_limit_with_increase(gas_estimate);
        let fee = self.calculate_fee(&input, &gas_limit).await?;

        Ok(TransactionLoadData {
            fee,
            metadata: input.metadata.clone(),
        })
    }

    pub async fn calculate_fee(&self, input: &TransactionLoadInput, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        if self.chain.is_opstack() {
            OptimismGasOracle::new(self.chain, self.clone()).calculate_fee(input, gas_limit).await
        } else {
            let extra_gas_limit = get_extra_fee_gas_limit(input)?;
            let gas_limit = gas_limit + &extra_gas_limit;
            let fee = input.gas_price.gas_price() * &gas_limit;

            Ok(TransactionFee::new_gas_price_type(input.gas_price.clone(), fee, gas_limit, HashMap::new()))
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_arbitrum_test_client, create_ethereum_test_client, create_smartchain_test_client, print_fee_rates, TEST_ADDRESS};
    use num_bigint::BigInt;
    use primitives::{Asset, Chain, FeePriority, GasPriceType, TransactionInputType, TransactionLoadInput};

    #[tokio::test]
    async fn test_ethereum_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Ethereum)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        println!("metadata: {:#?}", metadata);

        assert!(metadata.get_sequence()? > 0);
        assert_eq!(metadata.get_chain_id()?, "1");

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::SmartChain)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "0x0000000000000000000000000000000000000000".to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        println!("metadata: {:#?}", metadata);

        assert!(metadata.get_sequence()? > 0);
        assert_eq!(metadata.get_chain_id()?, "56");

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let input_type = TransactionInputType::Transfer(Asset::from_chain(Chain::Ethereum));

        let fee_rates = client.get_transaction_fee_rates(input_type).await?;

        print_fee_rates(fee_rates.clone());

        assert_eq!(fee_rates.len(), 3);

        for fee_rate in &fee_rates {
            assert!(fee_rate.gas_price_type.gas_price() > BigInt::from(0));
            assert!(fee_rate.gas_price_type.priority_fee() > BigInt::from(0));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_arbitrum_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_arbitrum_test_client();
        let input_type = TransactionInputType::Transfer(Asset::from_chain(Chain::Arbitrum));

        let fee_rates = client.get_transaction_fee_rates(input_type).await?;

        print_fee_rates(fee_rates.clone());

        assert_eq!(fee_rates.len(), 3);

        for fee_rate in &fee_rates {
            assert!(fee_rate.gas_price_type.gas_price() > BigInt::from(0));
            assert!(fee_rate.gas_price_type.priority_fee() > BigInt::from(0));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_transaction_fee_rates() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let input_type = TransactionInputType::Transfer(Asset::from_chain(Chain::SmartChain));

        let fee_rates = client.get_transaction_fee_rates(input_type).await?;

        print_fee_rates(fee_rates.clone());

        assert_eq!(fee_rates.len(), 3);

        for fee_rate in &fee_rates {
            //assert!(fee_rate.gas_price_type.gas_price() >= BigInt::from(100_000_000));
            assert!(fee_rate.gas_price_type.gas_price() < BigInt::from(1_000_000_000));
            assert!(fee_rate.gas_price_type.priority_fee() > BigInt::from(0));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_preload_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();

        let preload_input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Ethereum)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };
        let metadata = client.get_transaction_preload(preload_input.clone()).await?;

        let fee_rates = [FeeRate::new(
            FeePriority::Normal,
            GasPriceType::eip1559(BigInt::from(177554820), BigInt::from(100000000)),
        )];

        let gas_price = fee_rates.first().ok_or("No fee rates available")?.gas_price_type.clone();

        let load_input = TransactionLoadInput {
            input_type: preload_input.input_type,
            sender_address: preload_input.sender_address.clone(),
            destination_address: preload_input.destination_address.clone(),
            value: "100000000000000".to_string(),
            gas_price,
            memo: None,
            is_max_value: false,
            metadata,
        };

        let load_data = client.get_transaction_load(load_input).await?;

        println!("Transaction load data: {:#?}", load_data);

        assert!(load_data.fee.fee == BigInt::from(3728651220000u64));

        assert!(load_data.fee.gas_limit == BigInt::from(21000));

        assert!(load_data.metadata.get_sequence()? > 0);
        assert_eq!(load_data.metadata.get_chain_id()?, "1");

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_preload_transfer_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();

        let preload_input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_erc20()),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };
        let metadata = client.get_transaction_preload(preload_input.clone()).await?;

        let fee_rates = [FeeRate::new(
            FeePriority::Normal,
            GasPriceType::eip1559(BigInt::from(177554820), BigInt::from(100000000)),
        )];

        let gas_price = fee_rates.first().ok_or("No fee rates available")?.gas_price_type.clone();

        let load_input = TransactionLoadInput {
            input_type: preload_input.input_type,
            sender_address: preload_input.sender_address.clone(),
            destination_address: preload_input.destination_address.clone(),
            value: "1000000".to_string(),
            gas_price,
            memo: None,
            is_max_value: false,
            metadata,
        };

        let load_data = client.get_transaction_load(load_input).await?;

        println!("Token transfer load data: {:#?}", load_data);

        assert!(load_data.fee.fee > BigInt::from(0));
        assert!(load_data.fee.gas_limit > BigInt::from(0));

        assert!(load_data.fee.gas_limit > BigInt::from(21000));
        assert!(load_data.fee.gas_limit < BigInt::from(75000));

        assert!(load_data.metadata.get_sequence()? > 0);
        assert_eq!(load_data.metadata.get_chain_id()?, "1");

        Ok(())
    }
}
