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
            HyperCoreActionId::CDeposit { .. } | HyperCoreActionId::CWithdraw { .. } | HyperCoreActionId::TokenDelegate { .. } => {
                let updates = self.get_delegator_history(&request.sender_address).await?;
                Ok(transaction_state_mapper::map_transaction_state_staking_action(updates, action_id, request.id.clone()))
            }
            HyperCoreActionId::Nonce(nonce) => {
                let updates = self
                    .get_ledger_updates(
                        &request.sender_address,
                        nonce.saturating_sub(transaction_state_mapper::ACTION_HISTORY_QUERY_LOOKBACK_MS) as i64,
                    )
                    .await?;
                Ok(transaction_state_mapper::map_transaction_state_action(updates, action_id, request.id.clone()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use gem_client::testkit::MockClient;
    use primitives::{TransactionChange, TransactionState, TransactionStateRequest};

    #[tokio::test]
    async fn test_transaction_state_uses_delegator_history_for_c_withdraw() {
        let client = HyperCoreClient::new(MockClient::new().with_post(|_, body| {
            let payload: serde_json::Value = serde_json::from_slice(body).unwrap();
            assert_eq!(payload["type"], "delegatorHistory");
            Ok(
                r#"[{"time":1780078270596,"hash":"0x7b435a1210afafef7cbd043c84b8d402064e00f7aba2cec11f0c0564cfa389da","delta":{"withdrawal":{"amount":"0.03001423","phase":"initiated"}}}]"#
                    .as_bytes()
                    .to_vec(),
            )
        }));
        let request_id = "action:cWithdraw:3001423:1780078264489".to_string();
        let update = client
            .transaction_state(TransactionStateRequest {
                id: request_id.clone(),
                sender_address: "0x9EdcF9Ff72088DB8130C2512E5B4D3b5F34cEaF4".to_string(),
                created_at: Utc.timestamp_millis_opt(1780078264489).unwrap(),
                block_number: 0,
            })
            .await
            .unwrap();

        assert_eq!(update.state, TransactionState::Confirmed);
        assert_eq!(
            update.changes,
            vec![TransactionChange::HashChange {
                old: request_id,
                new: "0x7b435a1210afafef7cbd043c84b8d402064e00f7aba2cec11f0c0564cfa389da".to_string(),
            }]
        );
    }
}
