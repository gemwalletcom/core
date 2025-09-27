use crate::constants::TRANSACTION_TYPE_PAY;
use crate::models::{Transaction as AlgoTransaction, TransactionBroadcast};
use chrono::DateTime;
use primitives::{Transaction, TransactionState, TransactionType, chain::Chain};

pub fn map_transaction_broadcast(result: &TransactionBroadcast) -> Result<String, String> {
    if let Some(message) = &result.message {
        Err(message.clone())
    } else if let Some(hash) = &result.tx_id {
        Ok(hash.clone())
    } else {
        Err("Broadcast failed without specific error".to_string())
    }
}

pub fn map_transactions(transactions: Vec<AlgoTransaction>) -> Vec<Transaction> {
    transactions.into_iter().flat_map(map_transaction).collect::<Vec<Transaction>>()
}

pub fn map_transaction(transaction: AlgoTransaction) -> Option<Transaction> {
    let chain = Chain::Algorand;
    match transaction.transaction_type.as_str() {
        TRANSACTION_TYPE_PAY => Some(Transaction::new(
            transaction.id.clone(),
            chain.as_asset_id(),
            transaction.sender.clone().unwrap_or_default(),
            transaction.payment_transaction.clone()?.receiver.clone().unwrap_or_default(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            transaction.fee.unwrap_or_default().to_string(),
            chain.as_asset_id(),
            transaction.payment_transaction.clone()?.amount.unwrap_or_default().to_string(),
            transaction.clone().get_memo(),
            None,
            DateTime::from_timestamp(transaction.round_time, 0)?,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TransactionBroadcast;

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
}
