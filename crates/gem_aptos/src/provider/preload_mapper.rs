use std::error::Error;
use crate::model::Account;
use primitives::TransactionPreload;

pub fn map_transaction_preload(
    account: &Account,
) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
    Ok(TransactionPreload {
        block_hash: String::new(),
        block_number: 0,
        utxos: vec![],
        sequence: account.sequence_number,
        chain_id: String::new(),
        is_destination_address_exist: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Account;

    #[test]
    fn test_transaction_preload() {
        let account = Account {
            sequence_number: 42,
        };
        
        let result = map_transaction_preload(&account).unwrap();
        assert_eq!(result.sequence, 42);
    }
}