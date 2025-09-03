use std::error::Error;

use crate::fee_calculator::{get_fee_history_blocks, get_reward_percentiles};
#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionLoad;
#[cfg(feature = "rpc")]
use primitives::{FeeRate, TransactionInputType, TransactionLoadMetadata, TransactionPreloadInput};

use crate::provider::preload_mapper::{map_transaction_fee_rates, map_transaction_preload};
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
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{create_arbitrum_test_client, create_ethereum_test_client, create_smartchain_test_client, print_fee_rates, TEST_ADDRESS};
    use num_bigint::BigInt;
    use primitives::{Asset, Chain, TransactionInputType};

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
}
