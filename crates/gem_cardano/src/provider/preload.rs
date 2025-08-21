use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use std::error::Error;

use gem_client::Client;
use num_bigint::BigInt;
use primitives::{FeePriority, FeeRate, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, UTXO};

use super::preload_mapper;
use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for CardanoClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let utxos = self.get_utxos(&input.sender_address).await?;
        Ok(preload_mapper::map_transaction_preload(utxos, input))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: input.default_fee(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::regular(FeePriority::Normal, BigInt::from(1000000))])
    }

    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        Ok(CardanoClient::get_utxos(self, &address).await?.into_iter().map(UTXO::from).collect())
    }
}
