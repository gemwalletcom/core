use std::error::Error;

use primitives::graphql::GraphqlData;

use crate::models::transaction::TransactionBroadcast;
use crate::provider::transactions_mapper::map_transaction_broadcast;

pub(crate) fn map_transaction_broadcast_response(response: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    map_transaction_broadcast(response)
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_str::<GraphqlData<TransactionBroadcast>>(response)?;
    if response.errors.is_some() {
        return Err("Failed to broadcast transaction".into());
    }

    let hash = response
        .data
        .and_then(|data| data.submit_transaction)
        .map(|submit_transaction| submit_transaction.hash)
        .ok_or("Failed to broadcast transaction")?;

    map_transaction_broadcast_response(hash)
}
