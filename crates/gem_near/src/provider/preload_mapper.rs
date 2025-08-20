use crate::models::{NearAccountAccessKey, NearBlock};
use primitives::{TransactionLoadMetadata, TransactionPreloadInput};
use std::error::Error;

pub fn address_to_public_key(address: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let address_bytes = hex::decode(address)?;
    let encoded = bs58::encode(address_bytes).into_string();
    Ok(format!("ed25519:{}", encoded))
}

pub fn map_transaction_preload(
    _input: &TransactionPreloadInput,
    access_key: &NearAccountAccessKey,
    block: &NearBlock,
    is_destination_address_exist: bool,
) -> TransactionLoadMetadata {
    TransactionLoadMetadata::Near {
        sequence: (access_key.nonce + 1) as u64,
        block_hash: block.header.hash.clone(),
        is_destination_address_exist,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NearAccountAccessKey, NearBlock, NearBlockHeader};

    #[test]
    fn test_address_to_public_key() {
        let address = "051d30e6c78c4cf858389d62af5f703275450d318b85ff52a4ac963948cfdf95";
        let result = address_to_public_key(address).unwrap();
        assert!(result.starts_with("ed25519:"));
    }

    #[test]
    fn test_map_transaction_preload() {
        let input = TransactionPreloadInput {
            sender_address: "sender.near".to_string(),
            destination_address: "receiver.near".to_string(),
        };

        let access_key = NearAccountAccessKey { nonce: 116479371000026 };

        let block = NearBlock {
            header: NearBlockHeader {
                hash: "F45xbjXiyHn5noj1692RVqeuNC6X232qhKpvvPrv92iz".to_string(),
                height: 12345,
            },
        };

        let result = map_transaction_preload(&input, &access_key, &block, true);

        match result {
            TransactionLoadMetadata::Near { sequence, block_hash, is_destination_address_exist } => {
                assert_eq!(sequence, 116479371000027);
                assert_eq!(block_hash, "F45xbjXiyHn5noj1692RVqeuNC6X232qhKpvvPrv92iz");
                assert!(is_destination_address_exist);
            },
            _ => panic!("Expected Near metadata"),
        }
    }
}
