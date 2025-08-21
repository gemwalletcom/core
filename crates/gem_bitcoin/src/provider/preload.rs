use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::provider::preload_mapper::map_transaction_preload;
use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainPreload for BitcoinClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let utxos = self.get_utxos(&input.sender_address).await?;
        Ok(map_transaction_preload(utxos, input))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: primitives::TransactionFee::default(),
            metadata: input.metadata,
        })
    }
}
