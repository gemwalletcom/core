use crate::{
    Asset, Chain, GasPriceType, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra, TransferDataOutputAction, TransferDataOutputType,
    WalletConnectionSessionAppMetadata,
};
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

    pub fn mock_sign_data(chain: Chain, data: &str, output_type: TransferDataOutputType) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Generic(
                Asset::from_chain(chain),
                WalletConnectionSessionAppMetadata::mock(),
                TransferDataExtra {
                    data: Some(data.as_bytes().to_vec()),
                    output_type,
                    output_action: TransferDataOutputAction::Send,
                    ..Default::default()
                },
            ),
            sender_address: "test".into(),
            destination_address: "test".into(),
            value: "0".into(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::None,
        }
    }
}
