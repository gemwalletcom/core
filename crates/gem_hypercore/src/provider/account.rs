use async_trait::async_trait;
use chain_traits::ChainAccount;
use primitives::UTXO;
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainAccount for HyperCoreClient<C> {
    async fn get_utxos(&self, _address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
