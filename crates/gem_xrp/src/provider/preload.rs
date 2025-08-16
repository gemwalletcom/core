use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;
use futures;

use gem_client::Client;
use primitives::{TransactionPreload, TransactionPreloadInput};

use crate::rpc::client::XRPClient;

#[async_trait]
impl<C: Client> ChainPreload for XRPClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Send + Sync>> {
        let (account, latest_block) = futures::try_join!(
            self.get_account_info(&input.sender_address),
            self.get_ledger_current()
        )?;
        
        Ok(TransactionPreload::builder()
            .block_number(latest_block.ledger_current_index as i64)
            .sequence(account.sequence as i64)
            .build())
    }
}