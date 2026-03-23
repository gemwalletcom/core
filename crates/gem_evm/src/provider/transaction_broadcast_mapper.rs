use std::error::Error;

use gem_jsonrpc::types::JsonRpcResult;

pub fn map_transaction_broadcast_request(data: &str) -> String {
    if data.starts_with("0x") { data.to_string() } else { format!("0x{}", data) }
}

pub fn map_transaction_broadcast_response_from_str(response: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    Ok(serde_json::from_str::<JsonRpcResult<String>>(response)?.take()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_transaction_broadcast_request_encode() {
        assert_eq!(map_transaction_broadcast_request("123"), "0x123");
        assert_eq!(map_transaction_broadcast_request("0x123"), "0x123");
    }
}
