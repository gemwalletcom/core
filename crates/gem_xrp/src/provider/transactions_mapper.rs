use crate::rpc::model::TransactionBroadcast;
use std::error::Error;

pub fn map_transaction_broadcast(broadcast_result: &TransactionBroadcast) -> Result<String, Box<dyn Error + Sync + Send>> {
    if let Some(accepted) = broadcast_result.accepted
        && !accepted
    {
        if let Some(error_msg) = &broadcast_result.engine_result_message {
            return Err(format!("Transaction rejected: {}", error_msg).into());
        }
        return Err("Transaction was not accepted".into());
    }

    if let Some(hash) = &broadcast_result.hash {
        Ok(hash.clone())
    } else if let Some(tx_json) = &broadcast_result.tx_json {
        Ok(tx_json.hash.clone())
    } else {
        Err("Transaction broadcast failed - no hash returned".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{LedgerResult, TransactionBroadcast};

    #[test]
    fn test_map_transaction_broadcast_success() {
        let json_data = include_str!("../testdata/transaction_broadcast_success.json");
        let response: LedgerResult<TransactionBroadcast> = serde_json::from_str(json_data).expect("Failed to parse JSON");

        let result = map_transaction_broadcast(&response.result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "04F53F220DD1BCB7CCF279D66FFB986EA41383EFC9378CA1EBF1823D7C89264F");
    }

    #[test]
    fn test_map_transaction_broadcast_failed() {
        let json_data = include_str!("../testdata/transaction_broadcast_failed.json");
        let response: LedgerResult<TransactionBroadcast> = serde_json::from_str(json_data).expect("Failed to parse JSON");

        let result = map_transaction_broadcast(&response.result);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Transaction rejected: Ledger sequence too high.");
    }
}
