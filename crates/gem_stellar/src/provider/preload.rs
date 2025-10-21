use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    FeeOption, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::{models::AccountResult, rpc::client::StellarClient};

#[async_trait]
impl<C: Client> ChainTransactionLoad for StellarClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let destination_address = match input.input_type {
            TransactionInputType::Swap(_, _, swap_data) => swap_data.data.to.clone(),
            _ => input.destination_address.clone(),
        };
        let (sender_account, destination_exists) = futures::join!(self.get_account(input.sender_address.clone()), self.account_exists(&destination_address));
        match sender_account? {
            AccountResult::Found(account) => Ok(TransactionLoadMetadata::Stellar {
                sequence: account.sequence + 1,
                is_destination_address_exist: destination_exists?,
            }),
            AccountResult::NotFound => Err("Sender account not found".into()),
        }
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = if input.metadata.get_is_destination_address_exist()? {
            input.default_fee()
        } else {
            TransactionFee::new_from_fee_with_option(input.gas_price.gas_price(), FeeOption::TokenAccountCreation, BigInt::ZERO)
        };

        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;
        Ok(vec![
            FeeRate::new(FeePriority::Slow, GasPriceType::regular(fees.fee_charged.min)),
            FeeRate::new(FeePriority::Normal, GasPriceType::regular(fees.fee_charged.min)),
            FeeRate::new(FeePriority::Fast, GasPriceType::regular(fees.fee_charged.p95 * 2)),
        ])
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EMPTY_ADDRESS, create_test_client};
    use primitives::{Asset, Chain, TransactionInputType, TransactionPreloadInput};

    #[tokio::test]
    async fn test_stellar_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        assert!(metadata.get_sequence()? > 0);
        assert!(metadata.get_is_destination_address_exist()?);

        Ok(())
    }

    #[tokio::test]
    async fn test_stellar_get_transaction_preload_empty_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_EMPTY_ADDRESS.to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        assert!(metadata.get_sequence()? > 0);
        assert!(!metadata.get_is_destination_address_exist()?);

        Ok(())
    }

    #[tokio::test]
    async fn test_stellar_get_transaction_load() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let preload_input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };

        let metadata = client.get_transaction_preload(preload_input).await?;

        let load_input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
            value: "1000000".to_string(),
            gas_price: primitives::GasPriceType::regular(100),
            memo: None,
            is_max_value: false,
            metadata,
        };

        let load_data = client.get_transaction_load(load_input).await?;

        assert!(load_data.fee.fee > num_bigint::BigInt::from(0));
        assert!(load_data.metadata.get_sequence()? > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_stellar_get_transaction_load_empty_destination() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let preload_input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_EMPTY_ADDRESS.to_string(),
        };

        let metadata = client.get_transaction_preload(preload_input).await?;

        let load_input = TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_EMPTY_ADDRESS.to_string(),
            value: "1000000".to_string(),
            gas_price: primitives::GasPriceType::regular(100),
            memo: None,
            is_max_value: false,
            metadata,
        };

        let load_data = client.get_transaction_load(load_input).await?;

        assert!(load_data.fee.fee == num_bigint::BigInt::from(100));
        assert!(load_data.fee.options.contains_key(&primitives::FeeOption::TokenAccountCreation));
        assert_eq!(
            load_data.fee.options.get(&primitives::FeeOption::TokenAccountCreation),
            Some(&num_bigint::BigInt::from(0))
        );
        assert!(load_data.metadata.get_sequence()? > 0);

        Ok(())
    }
}
