use crate::{TransactionLoadMetadata, stake_type::TronStakeData};

impl TransactionLoadMetadata {
    pub fn mock_tron() -> Self {
        TransactionLoadMetadata::Tron {
            block_number: 0,
            block_version: 0,
            block_timestamp: 0,
            transaction_tree_root: "".to_string(),
            parent_hash: "".to_string(),
            witness_address: "".to_string(),
            stake_data: TronStakeData::Votes(vec![]),
        }
    }
}
