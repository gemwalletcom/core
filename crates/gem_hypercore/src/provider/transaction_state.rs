use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::{
    models::transaction_id::{HyperCoreActionId, HyperCoreTransactionId},
    provider::transaction_state_mapper,
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactionState for HyperCoreClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        self.transaction_state(request).await
    }
}

impl<C: Client> HyperCoreClient<C> {
    pub async fn transaction_state(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let id = HyperCoreTransactionId::parse(&request.id).ok_or("Invalid Hypercore transaction id")?;

        match id {
            HyperCoreTransactionId::Order(oid) => {
                let start_time = request.created_at.timestamp_millis() - transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS as i64;
                let fills = self.get_user_fills_by_time(&request.sender_address, start_time).await?;
                Ok(transaction_state_mapper::map_transaction_state_order(fills, oid, request.id))
            }
            HyperCoreTransactionId::Action(action_id) => self.action_state(&request, action_id).await,
        }
    }

    async fn action_state(&self, request: &TransactionStateRequest, action_id: HyperCoreActionId) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        match &action_id {
            HyperCoreActionId::Order(nonce) => {
                let start_time = nonce.saturating_sub(transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS) as i64;
                let fills = self.get_user_fills_by_time(&request.sender_address, start_time).await?;
                Ok(transaction_state_mapper::map_transaction_state_order_action(fills, *nonce, request.id.clone()))
            }
            HyperCoreActionId::Nonce(_) | HyperCoreActionId::CDeposit { .. } | HyperCoreActionId::CWithdraw { .. } | HyperCoreActionId::TokenDelegate { .. } => {
                let updates = self
                    .get_ledger_updates(
                        &request.sender_address,
                        action_id.nonce().saturating_sub(transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS) as i64,
                    )
                    .await?;
                Ok(transaction_state_mapper::map_transaction_state_action(updates, action_id, request.id.clone()))
            }
        }
    }
}
