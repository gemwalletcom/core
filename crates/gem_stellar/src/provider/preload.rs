use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use futures;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{
    transaction_load::FeeOption, FeePriority, FeeRate, TransactionFee, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for StellarClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let (sender_account, destination_result) = futures::join!(
            self.get_stellar_account(&input.sender_address),
            self.get_stellar_account(&input.destination_address)
        );

        Ok(TransactionLoadMetadata::Stellar {
            sequence: sender_account?.sequence + 1,
            is_destination_address_exist: destination_result.is_ok(),
        })
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let fee = if input.metadata.get_is_destination_address_exist()? {
            input.default_fee()
        } else {
            TransactionFee::new_from_fee_with_option(input.gas_price.gas_price(), FeeOption::TokenAccountCreation, BigInt::from(0))
        };
        Ok(TransactionLoadData { fee, metadata: input.metadata })
    }

    async fn get_transaction_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let fees = self.get_fees().await?;
        Ok(vec![
            FeeRate::regular(FeePriority::Slow, BigInt::from(fees.fee_charged.min)),
            FeeRate::regular(FeePriority::Normal, BigInt::from(fees.fee_charged.min)),
            FeeRate::regular(FeePriority::Fast, BigInt::from(fees.fee_charged.p95 * 2)),
        ])
    }
}
