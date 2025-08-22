use crate::rpc::model::AccountInfoResult;
use primitives::TransactionLoadMetadata;
use std::error::Error;

pub fn map_transaction_preload(account_result: AccountInfoResult) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
    if let Some(account_data) = account_result.account_data {
        Ok(TransactionLoadMetadata::Xrp {
            sequence: account_data.sequence,
            block_number: account_result.ledger_current_index,
        })
    } else {
        Err("Account not found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{AccountInfo, AccountInfoResult};

    #[test]
    fn test_map_transaction_preload_with_account_data() {
        let account_result = AccountInfoResult {
            account_data: Some(AccountInfo {
                balance: "1000000".to_string(),
                sequence: 12345,
                owner_count: 0,
                account: Some("rAccount123".to_string()),
                flags: Some(0),
                ledger_entry_type: Some("AccountRoot".to_string()),
            }),
            ledger_current_index: 67890,
        };

        let result = map_transaction_preload(account_result).unwrap();

        if let TransactionLoadMetadata::Xrp { sequence, block_number } = result {
            assert_eq!(sequence, 12345);
            assert_eq!(block_number, 67890);
        } else {
            panic!("Expected XRP metadata");
        }
    }

    #[test]
    fn test_map_transaction_preload_without_account_data() {
        let account_result = AccountInfoResult {
            account_data: None,
            ledger_current_index: 67890,
        };

        let result = map_transaction_preload(account_result);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Account not found");
    }
}
