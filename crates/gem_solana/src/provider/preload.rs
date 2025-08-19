use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput};

use crate::rpc::client::SolanaClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainPreload for SolanaClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let blockhash_result = self.get_latest_blockhash().await?;
        Ok(TransactionPreload::builder()
            .block_hash(blockhash_result.value.blockhash)
            .build())
    }
}