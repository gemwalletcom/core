use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput, UTXO};

use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainPreload for BitcoinClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let bitcoin_utxos = self.get_utxos(&input.sender_address).await?;
        
        let utxos: Vec<UTXO> = bitcoin_utxos.into_iter().map(|utxo| UTXO {
            transaction_id: utxo.txid,
            vout: utxo.vout,
            value: utxo.value,
            address: input.sender_address.clone(),
        }).collect();
        
        Ok(TransactionPreload {
            block_hash: String::new(),
            block_number: 0,
            utxos,
            sequence: 0,
            chain_id: String::new(),
            is_destination_address_exist: true,
        })
    }
}