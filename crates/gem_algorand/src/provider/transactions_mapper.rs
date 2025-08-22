use crate::rpc::model::{TransactionBroadcast, TransactionStatus};
use primitives::{TransactionChange, TransactionState, TransactionUpdate};

pub fn map_transaction_broadcast(result: &TransactionBroadcast) -> Result<String, String> {
    if let Some(message) = &result.message {
        Err(message.clone())
    } else if let Some(hash) = &result.tx_id {
        Ok(hash.clone())
    } else {
        Err("Broadcast failed without specific error".to_string())
    }
}

pub fn map_transaction_status(transaction: &TransactionStatus) -> TransactionUpdate {
    let confirmed_round = transaction.confirmed_round.unwrap_or(0);
    let state: TransactionState = if confirmed_round > 0 {
        TransactionState::Confirmed
    } else {
        TransactionState::Failed
    };

    let mut changes = Vec::new();
    if confirmed_round > 0 {
        changes.push(TransactionChange::BlockNumber(confirmed_round.to_string()));
    }

    TransactionUpdate { state, changes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{TransactionBroadcast, TransactionStatus};
    use primitives::{TransactionChange, TransactionState};

    #[test]
    fn test_map_transaction_broadcast_success() {
        let broadcast = TransactionBroadcast {
            tx_id: Some("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string()),
            message: None,
        };
        assert_eq!(
            map_transaction_broadcast(&broadcast),
            Ok("G4MBO3DS7ACGA3XF5XD5Y52ZVJL6ZYROTCVB2I3BQHBYHTPQ7VOA".to_string())
        );
    }

    #[test]
    fn test_map_transaction_broadcast_success_data() {
        let broadcast: TransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_success.json")).unwrap();
        assert_eq!(
            map_transaction_broadcast(&broadcast),
            Ok("LAEWXAG6FYFIEDAY76YQFKO46EIKEOIT4GTONUQFD6TL23XG45KQ".to_string())
        );
    }

    #[test]
    fn test_map_transaction_broadcast_error_data() {
        let broadcast: TransactionBroadcast = serde_json::from_str(include_str!("../../testdata/transaction_broadcast_error.json")).unwrap();
        assert_eq!(
            map_transaction_broadcast(&broadcast),
            Err("txgroup had 0 in fees, which is less than the minimum 1 * 1000".to_string())
        );
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let result = map_transaction_status(&TransactionStatus {
            confirmed_round: Some(52961610),
        });
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("52961610".to_string())]);
    }

    #[test]
    fn test_map_transaction_status_success_data() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_transfer_success.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::BlockNumber("52961610".to_string())]);
    }

    #[test]
    fn test_map_transaction_status_pending_data() {
        let status: TransactionStatus = serde_json::from_str(include_str!("../../testdata/transaction_transfer_pending.json")).unwrap();
        let result = map_transaction_status(&status);
        assert_eq!(result.state, TransactionState::Failed);
        assert_eq!(result.changes.len(), 0);
    }
}
