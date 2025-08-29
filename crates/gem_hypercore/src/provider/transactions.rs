use async_trait::async_trait;
use chain_traits::ChainTransactions;
use primitives::{
    BroadcastOptions, TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionState, TransactionStateRequest, TransactionUpdate,
};
use std::error::Error;

use gem_client::Client;

use crate::{provider::transactions_mapper::map_transaction_broadcast, rpc::client::HyperCoreClient};

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let response = self.exchange(serde_json::from_str(&data)?).await?;
        map_transaction_broadcast(response, data)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        self.transaction_state(request).await
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub async fn transaction_state(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let oid = match request.id.parse::<u64>() {
            Ok(id) => id,
            Err(_) => return Ok(TransactionUpdate::new_state(TransactionState::Confirmed)), // return confirmed if not an order id
        };
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
