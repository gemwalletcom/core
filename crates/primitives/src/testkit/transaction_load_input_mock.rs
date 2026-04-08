use super::signer_mock::{TEST_EVM_RECIPIENT, TEST_EVM_SENDER};
use crate::{
    Asset, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra, TransferDataOutputAction,
    TransferDataOutputType, WalletConnectionSessionAppMetadata,
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

    pub fn mock_aptos_token_transfer(token_id: &str) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::mock_with_params(
                Chain::Aptos,
                Some(token_id.to_string()),
                "USD Coin".to_string(),
                "USDC".to_string(),
                6,
                crate::AssetType::TOKEN,
            )),
            sender_address: "0x1".to_string(),
            destination_address: "0x2".to_string(),
            value: "1".to_string(),
            gas_price: GasPriceType::regular(BigInt::from(1u64)),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Aptos { sequence: 0, data: None },
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

    pub fn mock_evm(input_type: TransactionInputType, value: &str) -> Self {
        Self::mock_evm_with_metadata(input_type, value, TransactionLoadMetadata::mock_evm(0, 1))
    }

    pub fn mock_evm_with_metadata(input_type: TransactionInputType, value: &str, metadata: TransactionLoadMetadata) -> Self {
        TransactionLoadInput {
            input_type,
            sender_address: TEST_EVM_SENDER.to_string(),
            destination_address: TEST_EVM_RECIPIENT.to_string(),
            value: value.to_string(),
            gas_price: GasPriceType::eip1559(20_000_000_000u64, 1_000_000_000u64),
            memo: None,
            is_max_value: false,
            metadata,
        }
    }
}

impl SignerInput {
    pub fn mock_evm(input_type: TransactionInputType, value: &str, gas_limit: u64) -> Self {
        SignerInput::new(TransactionLoadInput::mock_evm(input_type, value), TransactionFee::mock_eip1559(gas_limit))
    }

    pub fn mock_evm_with_metadata(input_type: TransactionInputType, value: &str, gas_limit: u64, metadata: TransactionLoadMetadata) -> Self {
        SignerInput::new(
            TransactionLoadInput::mock_evm_with_metadata(input_type, value, metadata),
            TransactionFee::mock_eip1559(gas_limit),
        )
    }
}

impl TransactionLoadInput {
    pub fn mock_near(sender: &str, destination: &str, value: &str, sequence: u64, block_hash: &str) -> Self {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Near)),
            sender_address: sender.into(),
            destination_address: destination.into(),
            value: value.into(),
            gas_price: GasPriceType::regular(0),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Near {
                sequence,
                block_hash: block_hash.into(),
            },
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
