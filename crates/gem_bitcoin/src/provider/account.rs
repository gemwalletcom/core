use async_trait::async_trait;
use chain_traits::ChainAccount;
use primitives::UTXO;
use std::error::Error;

use gem_client::Client;

use super::account_mapper;
use crate::rpc::client::BitcoinClient;

#[async_trait]
impl<C: Client> ChainAccount for BitcoinClient<C> {
    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        let bitcoin_utxos = self.get_utxos(&address).await?;
        Ok(account_mapper::map_utxos(bitcoin_utxos, &address))
    }
}
