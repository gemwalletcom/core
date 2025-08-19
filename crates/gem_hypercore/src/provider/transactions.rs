use async_trait::async_trait;
use chain_traits::ChainTransactions;
use primitives::{TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.transaction_broadcast(data).await
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        self.transaction_state(request).await
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub async fn transaction_state(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let oid = request.id.parse::<u64>()?;
        let start_time = (request.created_at - 5) * 1000;
        let fills = self.get_user_fills_by_time(&request.sender_address, start_time).await?;
        let matching_fill = fills.iter().find(|fill| fill.oid == oid);

        match matching_fill {
            Some(fill) => {
                let pnl = fill.closed_pnl.parse::<f64>().unwrap_or(0.0);
                let price = fill.px.parse::<f64>().unwrap_or(0.0);

                let mut update = TransactionUpdate::new_state(TransactionState::Confirmed);
                update.changes = vec![
                    TransactionChange::HashChange {
                        old: request.id,
                        new: fill.hash.clone(),
                    },
                    TransactionChange::Metadata(TransactionMetadata::Perpetual(TransactionPerpetualMetadata { pnl, price })),
                ];
                Ok(update)
            }
            None => Ok(TransactionUpdate::new_state(TransactionState::Pending)),
        }
    }
}
