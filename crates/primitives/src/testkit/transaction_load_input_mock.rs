use crate::{Asset, Chain, GasPriceType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};
use num_bigint::BigInt;

impl TransactionLoadInput {
    pub fn mock() -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Sui)),
            sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            destination_address: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            value: "1000000000".to_string(),
            gas_price: GasPriceType::regular(BigInt::from(1000u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        }
    }

    pub fn mock_with_input_type(input_type: TransactionInputType) -> Self {
        TransactionLoadInput {
            input_type,
            sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            destination_address: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            value: "1000000000".to_string(),
            gas_price: GasPriceType::regular(BigInt::from(1000u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        }
    }
}
