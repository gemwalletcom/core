use std::error::Error;

use crate::models::transaction::PolkadotTransactionBroadcastResponse;

pub(crate) fn map_transaction_broadcast_response(response: PolkadotTransactionBroadcastResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(hash) = response.hash {
        Ok(hash)
    } else if let Some(error) = response.error {
        Err(format!("{}: {}", error, response.cause.unwrap_or_default()).into())
    } else {
        Err("Invalid broadcast response".into())
    }
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    map_transaction_broadcast_response(serde_json::from_str::<PolkadotTransactionBroadcastResponse>(response)?)
}
