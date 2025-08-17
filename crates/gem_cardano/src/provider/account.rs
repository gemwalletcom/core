use async_trait::async_trait;
use chain_traits::ChainAccount;
use std::error::Error;

use gem_client::Client;
use primitives::UTXO;

use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainAccount for CardanoClient<C> {
    async fn get_utxos(&self, address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        let cardano_utxos = self.get_utxos(&address).await?;

        let utxos = cardano_utxos
            .into_iter()
            .map(|utxo| UTXO {
                transaction_id: utxo.tx_hash,
                vout: utxo.index,
                value: utxo.value,
                address: utxo.address,
            })
            .collect();

        Ok(utxos)
    }
}
