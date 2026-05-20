use crate::{SolanaNftStandard, SolanaTokenProgramId, TransactionLoadMetadata, stake_type::TronStakeData};

impl TransactionLoadMetadata {
    pub fn mock_aptos() -> Self {
        TransactionLoadMetadata::Aptos { sequence: 0, data: None }
    }

    pub fn mock_osmosis() -> Self {
        TransactionLoadMetadata::Cosmos {
            account_number: 2_913_388,
            sequence: 10,
            chain_id: "osmosis-1".to_string(),
        }
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

    pub fn mock_solana(block_hash: &str) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: None,
            recipient_token_address: None,
            token_program: None,
            nft: None,
            block_hash: block_hash.to_string(),
        }
    }

    pub fn mock_solana_token(sender_token_address: Option<&str>, recipient_token_address: Option<&str>, token_program: Option<SolanaTokenProgramId>) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: sender_token_address.map(String::from),
            recipient_token_address: recipient_token_address.map(String::from),
            token_program,
            nft: None,
            block_hash: "11111111111111111111111111111111".to_string(),
        }
    }

    pub fn mock_solana_nft(sender_token_address: &str, token_program: SolanaTokenProgramId, nft: SolanaNftStandard) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: Some(sender_token_address.to_string()),
            recipient_token_address: None,
            token_program: Some(token_program),
            nft: Some(nft),
            block_hash: "11111111111111111111111111111111".to_string(),
        }
    }

    pub fn mock_solana_core_nft(collection: Option<&str>) -> Self {
        TransactionLoadMetadata::Solana {
            sender_token_address: None,
            recipient_token_address: None,
            token_program: None,
            nft: Some(SolanaNftStandard::Core {
                collection: collection.map(String::from),
            }),
            block_hash: "11111111111111111111111111111111".to_string(),
        }
    }
}
