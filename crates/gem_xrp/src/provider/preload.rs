use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionLoadMetadata, TransactionPreloadInput};

use crate::rpc::client::XRPClient;

#[async_trait]
impl<C: Client> ChainPreload for XRPClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        let account = self.get_account_info(&input.sender_address).await?;

        Ok(TransactionLoadMetadata::Xrp { sequence: account.sequence })
    }
}
