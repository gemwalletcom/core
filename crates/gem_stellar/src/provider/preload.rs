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
        let (sender_account, destination_exists) =
            futures::join!(self.get_account(input.sender_address.clone()), self.account_exists(&input.destination_address));
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
            TransactionFee::new_from_fee_with_option(input.gas_price.gas_price(), FeeOption::TokenAccountCreation, BigInt::from(0))
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
    use crate::provider::testkit::{TEST_ADDRESS, create_test_client};
    use primitives::{AssetType, Chain, TransactionInputType, TransactionPreloadInput};

    #[tokio::test]
    async fn test_stellar_get_transaction_preload() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let input = TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Chain::Stellar.new_asset("Stellar".to_string(), "XLM".to_string(), 7, AssetType::NATIVE)),
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: TEST_ADDRESS.to_string(),
        };

        let metadata = client.get_transaction_preload(input).await?;

        if let TransactionLoadMetadata::Stellar {
            sequence,
            is_destination_address_exist,
        } = metadata
        {
            assert!(sequence > 0, "Sequence should be greater than 0 for existing account");
            assert!(is_destination_address_exist, "Destination address should exist");
        } else {
            panic!("Expected Stellar metadata");
        }

        Ok(())
    }
}
