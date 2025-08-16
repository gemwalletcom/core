use crate::rpc::model::TransactionBroadcast;

pub fn map_transaction_broadcast(result: &TransactionBroadcast) -> Result<String, String> {
    if let Some(message) = &result.message {
        Err(message.clone())
    } else if let Some(hash) = &result.tx_id {
        Ok(hash.clone())
    } else {
        Err("Broadcast failed without specific error".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::TransactionBroadcast;

    #[test]
    fn test_map_transaction_broadcast_success() {
        let broadcast = TransactionBroadcast {
            tx_id: Some("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()),
            message: None,
        };
        
        let result = map_transaction_broadcast(&broadcast);
        assert_eq!(result, Ok("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()));
    }

    #[test]
    fn test_map_transaction_broadcast_error() {
        let broadcast = TransactionBroadcast {
            tx_id: None,
            message: Some("insufficient funds".to_string()),
        };
        
        let result = map_transaction_broadcast(&broadcast);
        assert_eq!(result, Err("insufficient funds".to_string()));
    }

    #[test]
    fn test_map_transaction_broadcast_no_response() {
        let broadcast = TransactionBroadcast {
            tx_id: None,
            message: None,
        };
        
        let result = map_transaction_broadcast(&broadcast);
        assert_eq!(result, Err("Broadcast failed without specific error".to_string()));
    }

    #[test]
    fn test_map_transaction_broadcast_both_fields() {
        let broadcast = TransactionBroadcast {
            tx_id: Some("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()),
            message: Some("warning message".to_string()),
        };
        
        // When both are present, message takes precedence (error case)
        let result = map_transaction_broadcast(&broadcast);
        assert_eq!(result, Err("warning message".to_string()));
    }
}