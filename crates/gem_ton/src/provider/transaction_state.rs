use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{
    provider::transaction_state_mapper::map_transaction_status,
    rpc::client::TonClient,
};

#[async_trait]
impl<C: Client> ChainTransactionState for TonClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transactions = self.get_transaction(request.id.clone()).await?;
        map_transaction_status(request, transactions)
    }
}
