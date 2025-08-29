use crate::models::response::{HyperCoreBroadcastResult, TransactionBroadcastResponse};
use std::error::Error;

pub fn map_transaction_broadcast(response: serde_json::Value, data: String) -> Result<String, Box<dyn Error + Sync + Send>> {
    let broadcast_response = serde_json::from_value::<TransactionBroadcastResponse>(response)?;
    match broadcast_response.into_result(data) {
        HyperCoreBroadcastResult::Success(result) => Ok(result),
        HyperCoreBroadcastResult::Error(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../tests/data/order_broadcast_filled.json")).unwrap();
        let result = map_transaction_broadcast(response, "test_hash".to_string()).unwrap();
        assert_eq!(result, "134896397196");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../tests/data/order_broadcast_error.json")).unwrap();
        let result = map_transaction_broadcast(response, "test_hash".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_map_transaction_broadcast_extra_agent_error() {
        let response: serde_json::Value = serde_json::from_str(include_str!("../../tests/data/transaction_broadcast_error_extra_agent.json")).unwrap();
        let result = map_transaction_broadcast(response, "test_hash".to_string());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Extra agent already used.");
    }
}
