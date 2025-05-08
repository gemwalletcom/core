use alloy_primitives::hex;
use bs58;

use crate::GemstoneError;

use super::{eip712::GemEIP712Message, sign_type::SignDigestType};

#[derive(Debug, uniffi::Record)]
pub struct SignMessage {
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}

#[derive(Debug, uniffi::Enum)]
pub enum MessagePreview {
    Text(String),
    EIP712(GemEIP712Message),
}

#[derive(Debug, uniffi::Object)]
pub struct SignMessageDecoder {
    pub message: SignMessage,
}

#[uniffi::export]
impl SignMessageDecoder {
    #[uniffi::constructor]
    pub fn new(message: SignMessage) -> Self {
        Self { message }
    }

    pub fn preview(&self) -> Result<MessagePreview, GemstoneError> {
        match self.message.sign_type {
            SignDigestType::Sign | SignDigestType::Eip191 => {
                let utf8_str = String::from_utf8(self.message.data.clone());
                let hex_str = hex::encode_prefixed(&self.message.data);
                let preview = utf8_str.unwrap_or(hex_str);
                Ok(MessagePreview::Text(preview))
            }
            SignDigestType::Eip712 => {
                let utf8_str = String::from_utf8(self.message.data.clone()).map_err(|_| GemstoneError::from("Invalid UTF-8 string for EIP712"))?;
                if utf8_str.is_empty() {
                    return Err(GemstoneError::from("Empty EIP712 message string"));
                }
                let message = GemEIP712Message::from_json(&utf8_str).map_err(|e| GemstoneError::from(format!("Invalid EIP712 message: {}", e)))?;
                Ok(MessagePreview::EIP712(message))
            }
            SignDigestType::Base58 => {
                let decoded = bs58::decode(&self.message.data).into_vec().unwrap_or_default();
                Ok(MessagePreview::Text(String::from_utf8_lossy(&decoded).to_string()))
            }
        }
    }

    pub fn get_result(&self, data: &[u8]) -> String {
        match self.message.sign_type {
            SignDigestType::Sign | SignDigestType::Eip191 | SignDigestType::Eip712 => hex::encode_prefixed(data),
            SignDigestType::Base58 => bs58::encode(data).into_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::sign_type::SignDigestType;
    use alloy_primitives::hex;
    use gem_evm::eip712::EIP712TypedValue;

    #[test]
    fn test_eip191() {
        let data = b"test".to_vec();
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip191,
            data,
        });
        match decoder.preview() {
            Ok(MessagePreview::Text(preview)) => assert_eq!(preview, "test"),
            _ => panic!("Unexpected preview result"),
        }
    }

    #[test]
    fn test_eip191_hex_value() {
        // 0x74657374 corresponds to "test" in UTF-8
        let data = hex::decode("74657374").expect("Invalid hex string");
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip191,
            data,
        });
        match decoder.preview() {
            Ok(MessagePreview::Text(preview)) => assert_eq!(preview, "test"),
            _ => panic!("Unexpected preview result"),
        }
    }

    #[test]
    fn test_eip191_non_utf8_hex_value() {
        // 0xdeadbeef is not valid UTF-8
        let data = hex::decode("deadbeef").expect("Invalid hex string");
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip191,
            data,
        });
        match decoder.preview() {
            // Since 0xdeadbeef is not valid UTF-8, preview should show the hex representation
            Ok(MessagePreview::Text(preview)) => assert_eq!(preview, "0xdeadbeef"),
            _ => panic!("Unexpected preview result"),
        }
    }

    #[test]
    fn test_base58() {
        let data = b"StV1DL6CwTryKyV".to_vec(); // Base58 encoded form of "hello world"
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Base58,
            data: data.clone(),
        });

        match decoder.preview() {
            Ok(MessagePreview::Text(preview)) => assert_eq!(preview, "hello world"),
            _ => panic!("Unexpected preview result for base58"),
        }

        let result_data = b"StV1DL6CwTryKyV"; // Data to pass to get_result, mimicking Swift test
        let result = decoder.get_result(result_data);

        assert_eq!(result, "3LRFsmWKLfsR7G5PqjytR");
    }

    #[test]
    fn test_eip712() {
        // Load the JSON content from the file
        let json_data_str = include_str!("test/uniswap_permit2.json");
        let data = json_data_str.as_bytes().to_vec();

        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip712,
            data: data.clone(),
        });

        match decoder.preview() {
            Ok(MessagePreview::EIP712(message)) => {
                assert_eq!(message.domain.name, "Permit2");
                assert_eq!(message.domain.chain_id, 1);
                assert_eq!(message.domain.verifying_contract.to_lowercase(), "0x000000000022d473030f116ddee9f6b43ac78ba3");
                assert_eq!(message.message.len(), 3);
                assert_eq!(message.message[0].name, "details");

                match &message.message[0].value {
                    EIP712TypedValue::Struct { fields } => {
                        assert_eq!(fields.len(), 4); // token, amount, expiration, nonce

                        // 1.1 token (address)
                        assert_eq!(fields[0].name, "token");
                        match &fields[0].value {
                            EIP712TypedValue::Address { value } => assert_eq!(value, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                            _ => panic!("Incorrect type for details.token"),
                        }
                        // 1.2 amount (uint160 - parsed as Uint256 for now)
                        // We parse uint160 as Uint256 { value: String } because the JSON value is a string.
                        assert_eq!(fields[1].name, "amount");
                        match &fields[1].value {
                            EIP712TypedValue::Uint256 { value } => assert_eq!(value, "1461501637330902918203684832716283019655932542975"),
                            _ => panic!("Incorrect type for details.amount"),
                        }
                        // 1.3 expiration (uint48 - parsed as Uint256 for now)
                        assert_eq!(fields[2].name, "expiration");
                        match &fields[2].value {
                            EIP712TypedValue::Uint256 { value } => assert_eq!(value, "1732780554"),
                            _ => panic!("Incorrect type for details.expiration"),
                        }
                        // 1.4 nonce (uint48 - parsed as Uint256 for now)
                        assert_eq!(fields[3].name, "nonce");
                        match &fields[3].value {
                            EIP712TypedValue::Uint256 { value } => assert_eq!(value, "0"),
                            _ => panic!("Incorrect type for details.nonce"),
                        }
                    }
                    _ => panic!("Incorrect type for details field"),
                }

                assert_eq!(message.message[1].name, "spender");
                match &message.message[1].value {
                    EIP712TypedValue::Address { value } => {
                        assert_eq!(value.to_lowercase(), "0x3fc91a3afd70395cd496c647d5a6cc9d4b2b7fad");
                    }
                    _ => panic!("Expected spender field to be an Address"),
                }

                assert_eq!(message.message[2].name, "sigDeadline");
                match &message.message[2].value {
                    EIP712TypedValue::Uint256 { value } => {
                        assert_eq!(value, "1730190354");
                    }
                    _ => panic!("Expected sigDeadline field to be a Uint256"),
                }
            }
            Ok(_) => panic!("Expected EIP712 preview, got Text"),
            Err(e) => panic!("Preview failed for EIP712: {:?}", e),
        }

        // Test get_result: Should just hex encode the input data for EIP712
        let result_data = b"some_bytes_to_encode";
        let result = decoder.get_result(result_data);

        assert_eq!(result, hex::encode_prefixed(result_data));
    }
}
