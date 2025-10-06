use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionChange, TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::{models::action::ExchangeRequest, provider::transaction_state_mapper, rpc::client::HyperCoreClient};

#[async_trait]
impl<C: Client> ChainTransactionState for HyperCoreClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        self.transaction_state(request).await
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub async fn transaction_state(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        match request.id.parse::<u64>() {
            Ok(oid) => {
                let start_time = (request.created_at - 5) * 1000;
                let fills = self.get_user_fills_by_time(&request.sender_address, start_time).await?;
                Ok(transaction_state_mapper::map_transaction_state_order(fills, oid, request.id))
            }
            Err(_) => self.action_state(&request).await,
        }
    }

    async fn action_state(&self, request: &TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let original: ExchangeRequest = serde_json::from_str(&request.id)?;
        let hash = self.get_tx_hash_by_nonce(&request.sender_address, original.nonce).await;

        let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);
        if let Ok(hash) = hash {
            update.changes = vec![TransactionChange::HashChange {
                old: request.id.clone(),
                new: hash,
            }];
        }
        Ok(update)
    }
}
