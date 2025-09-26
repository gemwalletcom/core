use primitives::{TransactionState, TransactionUpdate};

use crate::models::TransactionResponse;

pub fn map_transaction_status(transaction: TransactionResponse) -> TransactionUpdate {
    let state = if transaction.tx_response.code == 0 { TransactionState::Confirmed } else { TransactionState::Reverted };

    TransactionUpdate::new_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TransactionBody, TransactionResponseData, TransactionResponseTx};

    fn create_response(code: i64) -> TransactionResponse {
        TransactionResponse {
            tx: TransactionResponseTx {
                body: TransactionBody {
                    memo: String::new(),
                    messages: vec![],
                },
                auth_info: None,
            },
            tx_response: TransactionResponseData {
                code,
                txhash: "hash".to_string(),
                events: vec![],
                timestamp: String::new(),
            },
        }
    }

    #[test]
    fn test_map_transaction_status_confirmed() {
        let response = create_response(0);
        let update = map_transaction_status(response);
        assert_eq!(update.state, TransactionState::Confirmed);
    }

    #[test]
    fn test_map_transaction_status_reverted() {
        let response = create_response(1);
        let update = map_transaction_status(response);
        assert_eq!(update.state, TransactionState::Reverted);
    }
}
