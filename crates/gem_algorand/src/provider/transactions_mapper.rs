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
    fn test_map_transaction_broadcast() {
        let broadcast = TransactionBroadcast {
            tx_id: Some("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()),
            message: None,
        };
        
        let result = map_transaction_broadcast(&broadcast);
        assert_eq!(result, Ok("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()));
    }
}