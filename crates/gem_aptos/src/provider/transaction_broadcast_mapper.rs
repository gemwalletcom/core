use std::error::Error;

use primitives::decode_hex;

use crate::models::{SubmitTransactionBcsRequest, TransactionResponse};
use crate::provider::transactions_mapper::map_transaction_broadcast;

pub fn map_transaction_broadcast_request(data: &str) -> Result<Vec<u8>, Box<dyn Error + Sync + Send>> {
    let request = serde_json::from_str::<SubmitTransactionBcsRequest>(data)?;
    Ok(decode_hex(&request.bcs)?)
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let response = serde_json::from_str::<TransactionResponse>(response)?;
    map_transaction_broadcast(&response)
}
