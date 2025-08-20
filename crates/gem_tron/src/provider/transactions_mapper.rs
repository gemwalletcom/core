use crate::rpc::model::TronTransactionBroadcast;
use std::error::Error;

pub fn map_transaction_broadcast(response: &TronTransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(txid) = &response.txid {
        Ok(txid.clone())
    } else if let (Some(code), Some(message)) = (&response.code, &response.message) {
        Err(format!("Broadcast failed [{}]: {}", code, message).into())
    } else {
        Err("Transaction broadcast failed with unknown error".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::TronTransactionBroadcast;

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = TronTransactionBroadcast {
            txid: Some("ABC123".to_string()),
            code: None,
            message: None,
        };

        assert_eq!(map_transaction_broadcast(&response).unwrap(), "ABC123");
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let response = TronTransactionBroadcast {
            txid: None,
            code: Some("INVALID_TX".to_string()),
            message: Some("Transaction validation failed".to_string()),
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Broadcast failed [INVALID_TX]: Transaction validation failed");
    }

    #[test]
    fn test_map_transaction_broadcast_unknown_error() {
        let response = TronTransactionBroadcast {
            txid: None,
            code: None,
            message: None,
        };

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction broadcast failed with unknown error");
    }
}