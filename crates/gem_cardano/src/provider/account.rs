use async_trait::async_trait;
use chain_traits::ChainAccount;
use std::error::Error;

use gem_client::Client;
use primitives::UTXO;

use super::account_mapper;
use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainAccount for CardanoClient<C> {
    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        let cardano_utxos = self.get_utxos(&address).await?;
        Ok(account_mapper::map_utxos(cardano_utxos))
    }
}
