use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput, UTXO};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainPreload for BitcoinClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let bitcoin_utxos = self.get_utxos(&input.sender_address).await?;

        let utxos: Vec<UTXO> = bitcoin_utxos
            .into_iter()
            .map(|utxo| UTXO {
                transaction_id: utxo.txid,
                vout: utxo.vout,
                value: utxo.value,
                address: input.sender_address.clone(),
            })
            .collect();

        Ok(TransactionPreload::builder().utxos(utxos).build())
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
