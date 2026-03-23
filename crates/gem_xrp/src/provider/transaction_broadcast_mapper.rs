use std::error::Error;

use gem_jsonrpc::types::JsonRpcResult;

use crate::models::rpc::TransactionBroadcast;
use crate::provider::transactions_mapper::map_transaction_broadcast;

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_str::<JsonRpcResult<TransactionBroadcast>>(response)?.take()?;
    map_transaction_broadcast(&response)
}
