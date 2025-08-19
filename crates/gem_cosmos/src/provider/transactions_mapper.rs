use crate::models::transaction::CosmosBroadcastResponse;
use std::error::Error;

pub fn map_transaction_broadcast(response: &CosmosBroadcastResponse) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(tx_response) = &response.tx_response {
        if tx_response.code != 0 {
            Err(tx_response.raw_log.clone().into())
        } else {
            Ok(tx_response.txhash.clone())
        }
    } else if let Some(message) = &response.message {
        Err(format!("Broadcast error: {}", message).into())
    } else {
        Err("Unknown broadcast error".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{CosmosBroadcastResponse, CosmosTransactionResult};

    #[test]
    fn test_map_transaction_broadcast_success() {
        let response = CosmosBroadcastResponse {
            tx_response: Some(CosmosTransactionResult {
                txhash: "ABC123".to_string(),
                code: 0,
                raw_log: "".to_string(),
            }),
            code: None,
            message: None,
        };

        assert_eq!(map_transaction_broadcast(&response).unwrap(), "ABC123");
    }

    #[test]
    fn test_map_transaction_broadcast_failed() {
        let response: CosmosBroadcastResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_failed.json")).unwrap();

        assert!(map_transaction_broadcast(&response).is_err());
    }

    #[test]
    fn test_map_transaction_failed() {
        let response: CosmosBroadcastResponse = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_failed.json")).unwrap();

        let result = map_transaction_broadcast(&response);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "signature verification failed; please verify account number (1343971) and chain-id (cosmoshub-4): (unable to verify single signer signature): unauthorized");
    }
}
