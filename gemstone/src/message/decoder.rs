use alloy_primitives::hex;
use bs58;

use crate::GemstoneError;
use gem_evm::eip712::hash_eip712_json;

use super::{
    eip712::GemEIP712Message,
    sign_type::{SignDigestType, SignMessage},
};

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

    pub fn hash(&self) -> Vec<u8> {
        match self.message.sign_type {
            SignDigestType::Sign => self.message.data.clone(),
            SignDigestType::Eip191 => {
                let prefix = "\x19Ethereum Signed Message:\n";
                let mut data = prefix.as_bytes().to_vec();
                data.extend_from_slice(&self.message.data);
                data
            }
            SignDigestType::Eip712 => {
                if let Ok(value) = serde_json::from_slice(&self.message.data) {
                    hash_eip712_json(value).unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            SignDigestType::Base58 => {
                // Check if the data is a valid base58 string in utf8
                if let Ok(string_data) = String::from_utf8(self.message.data.clone()) {
                    if bs58::decode(string_data.as_bytes()).into_vec().is_ok() {
                        return self.message.data.clone();
                    }
                }
                Vec::new()
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
    fn test_eip712_hash() {
        let json = serde_json::json!({
            "types": {
                "EIP712Domain": [
                    {
                        "name": "name",
                        "type": "string"
                    },
                    {
                        "name": "version",
                        "type": "string"
                    },
                    {
                        "name": "chainId",
                        "type": "uint256"
                    },
                    {
                        "name": "verifyingContract",
                        "type": "address"
                    }
                ],
                "OrderComponents": [
                    {
                        "name": "offerer",
                        "type": "address"
                    },
                    {
                        "name": "zone",
                        "type": "address"
                    },
                    {
                        "name": "offer",
                        "type": "OfferItem[]"
                    },
                    {
                        "name": "startTime",
                        "type": "uint256"
                    },
                    {
                        "name": "endTime",
                        "type": "uint256"
                    },
                    {
                        "name": "zoneHash",
                        "type": "bytes32"
                    },
                    {
                        "name": "salt",
                        "type": "uint256"
                    },
                    {
                        "name": "conduitKey",
                        "type": "bytes32"
                    },
                    {
                        "name": "counter",
                        "type": "uint256"
                    }
                ],
                "OfferItem": [
                    {
                        "name": "token",
                        "type": "address"
                    }
                ],
                "ConsiderationItem": [
                    {
                        "name": "token",
                        "type": "address"
                    },
                    {
                        "name": "identifierOrCriteria",
                        "type": "uint256"
                    },
                    {
                        "name": "startAmount",
                        "type": "uint256"
                    },
                    {
                        "name": "endAmount",
                        "type": "uint256"
                    },
                    {
                        "name": "recipient",
                        "type": "address"
                    }
                ]
            },
            "primaryType": "OrderComponents",
            "domain": {
                "name": "Seaport",
                "version": "1.1",
                "chainId": "1",
                "verifyingContract": "0x00000000006c3852cbEf3e08E8dF289169EdE581"
            },
            "message": {
                "offerer": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266",
                "offer": [
                    {
                        "token": "0xA604060890923Ff400e8c6f5290461A83AEDACec"
                    }
                ],
                "startTime": "1658645591",
                "endTime": "1659250386",
                "zone": "0x004C00500000aD104D7DBd00e3ae0A5C00560C00",
                "zoneHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "salt": "16178208897136618",
                "conduitKey": "0x0000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000",
                "totalOriginalConsiderationItems": "2",
                "counter": "0"
            }
        });

        let hash = hash_eip712_json(json).unwrap();
        assert_eq!(hex::encode(&hash), "0b8aa9f3712df0034bc29fe5b24dd88cfdba02c7f499856ab24632e2969709a8",);
    }
}
