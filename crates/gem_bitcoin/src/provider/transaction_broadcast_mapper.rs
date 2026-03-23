use std::error::Error;

use crate::models::BitcoinTransactionBroacastResult;
use crate::provider::transactions_mapper::map_transaction_broadcast;

pub(crate) fn map_transaction_broadcast_response(response: BitcoinTransactionBroacastResult) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(error) = response.error {
        return Err(error.message().into());
    }

    map_transaction_broadcast(response.result.ok_or("unknown hash")?)
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    map_transaction_broadcast_response(serde_json::from_str::<BitcoinTransactionBroacastResult>(response)?)
}
