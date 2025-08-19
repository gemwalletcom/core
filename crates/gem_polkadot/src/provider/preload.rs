use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput};

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainPreload for PolkadotClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        Ok(TransactionPreload::default())
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: primitives::TransactionFee::default(),
            metadata: primitives::transaction_load::TransactionLoadMetadata::Polkadot {
                sequence: input.sequence,
            },
        })
    }
}