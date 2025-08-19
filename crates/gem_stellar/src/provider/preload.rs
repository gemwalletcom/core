use async_trait::async_trait;
use chain_traits::ChainPreload;
use futures;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionFee, TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainPreload for StellarClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let (sender_account, destination_result) = futures::join!(
            self.get_stellar_account(&input.sender_address),
            self.get_stellar_account(&input.destination_address)
        );

        let current_sequence: i64 = sender_account?.sequence.parse().unwrap_or(0);
        let sequence = (current_sequence + 1) as u64;
        let is_destination_address_exist = destination_result.is_ok();

        Ok(TransactionPreload::builder()
            .sequence(sequence)
            .is_destination_address_exist(is_destination_address_exist)
            .build())
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::default(),
            metadata: TransactionLoadMetadata::Stellar {
                sequence: input.sequence,
            },
        })
    }
}
