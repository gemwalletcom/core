use std::error::Error;

use gem_jsonrpc::types::JsonRpcResult;

use crate::models::transaction::BroadcastResult;
use crate::provider::transactions_mapper::map_transaction_broadcast;

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_str::<JsonRpcResult<BroadcastResult>>(response)?.take()?;
    map_transaction_broadcast(&response)
}
