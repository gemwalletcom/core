use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{
    TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionStateRequest, TransactionUpdate,
};
use std::error::Error;

use gem_client::Client;

use crate::{models::action::ExchangeRequest, rpc::client::HyperCoreClient};

#[async_trait]
impl<C: Client> ChainTransactionState for HyperCoreClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        self.transaction_state(request).await
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub async fn transaction_state(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        match request.id.parse::<u64>() {
            Ok(id) => self.order_state(&request, id).await,
            Err(_) => self.action_state(&request).await,
        }
    }

    async fn order_state(&self, request: &TransactionStateRequest, oid: u64) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let start_time = (request.created_at - 5) * 1000;
        let fills = self.get_user_fills_by_time(&request.sender_address, start_time).await?;
        let matching_fill = fills.iter().find(|fill| fill.oid == oid);

        match matching_fill {
            Some(fill) => {
                let pnl = fill.closed_pnl;
                let price = fill.px;

                let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);
                update.changes = vec![
                    TransactionChange::HashChange {
                        old: request.id.clone(),
                        new: fill.hash.clone(),
                    },
                    TransactionChange::Metadata(TransactionMetadata::Perpetual(TransactionPerpetualMetadata { pnl, price })),
                ];
                Ok(update)
            }
            None => Ok(TransactionUpdate::new_state(TransactionState::Pending)),
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
