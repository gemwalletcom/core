use crate::models::Account;
use primitives::TransactionLoadMetadata;
use std::error::Error;

pub fn map_transaction_preload(account: &Account) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
    Ok(TransactionLoadMetadata::Aptos {
        sequence: account.sequence_number,
        data: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Account;

    #[test]
    fn test_transaction_preload() {
        let account = Account { sequence_number: 42 };

        let result = map_transaction_preload(&account).unwrap();
        match result {
            TransactionLoadMetadata::Aptos { sequence, .. } => assert_eq!(sequence, 42),
            _ => panic!("Expected Aptos metadata"),
        }
    }
}
