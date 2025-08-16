use async_trait::async_trait;
use chain_traits::{ChainPreload, ChainBalances};
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput};

use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainPreload for StellarClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let is_destination_address_exist = if input.destination_address.is_empty() {
            true
        } else {
            match self.get_balance_coin(input.destination_address.clone()).await {
                Ok(_) => true,
                Err(_) => false,
            }
        };

        Ok(TransactionPreload {
            block_hash: String::new(),
            block_number: 0,
            utxos: vec![],
            sequence: 0,
            chain_id: String::new(),
            is_destination_address_exist,
        })
    }
}