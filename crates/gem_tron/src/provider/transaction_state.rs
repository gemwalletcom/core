use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{
    provider::transaction_state_mapper::map_transaction_status,
    rpc::client::TronClient,
};

#[async_trait]
impl<C: Client + Clone> ChainTransactionState for TronClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let receipt = self.get_transaction_reciept(request.id).await?;
        Ok(map_transaction_status(&receipt))
    }
}
