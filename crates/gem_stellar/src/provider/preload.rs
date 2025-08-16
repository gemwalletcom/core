use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;
use futures;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput};

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainPreload for StellarClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let (sender_account, destination_result) = futures::join!(
            self.get_stellar_account(&input.sender_address),
            self.get_stellar_account(&input.destination_address)
        );
        
        let current_sequence: i64 = sender_account?.sequence.parse().unwrap_or(0);
        let sequence = current_sequence + 1;
        let is_destination_address_exist = destination_result.is_ok();

        Ok(TransactionPreload::builder()
            .sequence(sequence)
            .is_destination_address_exist(is_destination_address_exist)
            .build())
    }
}