use crate::models::{AccountAccessKey, Block};
use primitives::TransactionLoadMetadata;
use std::error::Error;

pub fn address_to_public_key(address: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let address_bytes = hex::decode(address)?;
    let encoded = bs58::encode(address_bytes).into_string();
    Ok(format!("ed25519:{}", encoded))
}

pub fn map_transaction_preload(access_key: &AccountAccessKey, block: &Block) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Near {
        sequence: (access_key.nonce + 1) as u64,
        block_hash: block.header.hash.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountAccessKey, Block, BlockHeader};

    #[test]
    fn test_address_to_public_key() {
        let address = "051d30e6c78c4cf858389d62af5f703275450d318b85ff52a4ac963948cfdf95";
        let result = address_to_public_key(address).unwrap();
        assert!(result.starts_with("ed25519:"));
    }

    #[test]
    fn test_map_transaction_preload() {
        let access_key = AccountAccessKey { nonce: 116479371000026 };

        let block = Block {
            header: BlockHeader {
                hash: "F45xbjXiyHn5noj1692RVqeuNC6X232qhKpvvPrv92iz".to_string(),
                height: 12345,
            },
        };

        let result = map_transaction_preload(&access_key, &block);

        match result {
            TransactionLoadMetadata::Near { sequence, block_hash } => {
                assert_eq!(sequence, 116479371000027);
                assert_eq!(block_hash, "F45xbjXiyHn5noj1692RVqeuNC6X232qhKpvvPrv92iz");
            }
            _ => panic!("Expected Near metadata"),
        }
    }
}
