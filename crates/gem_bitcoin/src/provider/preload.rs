use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::rpc::client::BitcoinClient;
use crate::provider::preload_mapper::map_transaction_preload;

#[async_trait]
impl<C: Client> ChainPreload for BitcoinClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let utxos = self.get_utxos(&input.sender_address).await?;
        Ok(map_transaction_preload(utxos, input))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: primitives::TransactionFee::default(),
            metadata: TransactionLoadMetadata::Bitcoin { 
                utxos: input.utxos 
            },
        })
    }
}
