use std::error::Error;

use gem_jsonrpc::types::JsonRpcResult;

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    Ok(serde_json::from_str::<JsonRpcResult<String>>(response)?.take()?)
}
