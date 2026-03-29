use crate::{Asset, Chain, TransactionInputType, TransactionPreloadInput};

impl TransactionPreloadInput {
    pub fn mock() -> Self {
        TransactionPreloadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Aptos)),
            sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            destination_address: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        }
    }

    pub fn mock_with_input_type(input_type: TransactionInputType) -> Self {
        TransactionPreloadInput {
            input_type,
            sender_address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            destination_address: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
        }
    }
}
