use crate::{TransactionLoadMetadata, stake_type::TronStakeData};

impl TransactionLoadMetadata {
    pub fn mock_aptos() -> Self {
        TransactionLoadMetadata::Aptos { sequence: 0, data: None }
    }

    pub fn mock_evm(nonce: u64, chain_id: u64) -> Self {
        TransactionLoadMetadata::Evm {
            nonce,
            chain_id,
            contract_call: None,
        }
    }

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

    pub fn mock_ton(sequence: u64) -> Self {
        TransactionLoadMetadata::Ton {
            sender_token_address: None,
            recipient_token_address: None,
            sequence,
        }
    }

    pub fn mock_ton_jetton(sequence: u64, sender_token_address: &str) -> Self {
        TransactionLoadMetadata::Ton {
            sender_token_address: Some(sender_token_address.to_string()),
            recipient_token_address: None,
            sequence,
        }
    }
}
