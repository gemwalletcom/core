use std::error::Error;

use gem_jsonrpc::types::JsonRpcResult;

use crate::models::SuiBroadcastTransaction;

pub fn map_transaction_broadcast_request(data: &str) -> Result<(String, String), Box<dyn Error + Sync + Send>> {
    let parts = data.split_once('_').ok_or("Invalid transaction data format. Expected format: data_signature")?;
    Ok((parts.0.to_string(), parts.1.to_string()))
}

pub(crate) fn map_transaction_broadcast_response(response: SuiBroadcastTransaction) -> Result<String, Box<dyn Error + Sync + Send>> {
    Ok(response.digest)
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    map_transaction_broadcast_response(serde_json::from_str::<JsonRpcResult<SuiBroadcastTransaction>>(response)?.take()?)
}
