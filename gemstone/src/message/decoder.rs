use alloy_primitives::{eip191_hash_message, hex};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bs58;
use std::borrow::Cow;
use sui_types::{Ed25519PublicKey, Ed25519Signature, PersonalMessage};

use super::{
    eip712::GemEIP712Message,
    sign_type::{SignDigestType, SignMessage},
};
use crate::GemstoneError;
use gem_evm::eip712::eip712_hash_message;
const SIGNATURE_LENGTH: usize = 65;
const RECOVERY_ID_INDEX: usize = SIGNATURE_LENGTH - 1;
const ETHEREUM_RECOVERY_ID_OFFSET: u8 = 27;
const SUI_PERSONAL_SIGNATURE_LEN: usize = 1 + Ed25519Signature::LENGTH + Ed25519PublicKey::LENGTH;

#[derive(Debug, PartialEq, uniffi::Enum)]
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
            SignDigestType::Sign | SignDigestType::Eip191 | SignDigestType::SuiPersonalMessage => {
                let utf8_str = String::from_utf8(self.message.data.clone());
                let preview = utf8_str.unwrap_or(hex::encode_prefixed(&self.message.data));
                Ok(MessagePreview::Text(preview))
            }
            SignDigestType::Eip712 => {
                let utf8_str = String::from_utf8(self.message.data.clone()).map_err(|_| GemstoneError::from("Invalid UTF-8 string for EIP712"))?;
                if utf8_str.is_empty() {
                    return Err(GemstoneError::from("Empty EIP712 message string"));
                }
                let message = GemEIP712Message::from_json(&utf8_str).map_err(|e| GemstoneError::from(format!("Invalid EIP712 message: {e}")))?;
                Ok(MessagePreview::EIP712(message))
            }
            SignDigestType::Base58 => {
                let decoded = bs58::decode(&self.message.data).into_vec().unwrap_or_default();
                Ok(MessagePreview::Text(String::from_utf8_lossy(&decoded).to_string()))
            }
        }
    }

    pub fn plain_preview(&self) -> String {
        match self.message.sign_type {
            SignDigestType::Sign | SignDigestType::Eip191 | SignDigestType::Base58 | SignDigestType::SuiPersonalMessage => match self.preview() {
                Ok(MessagePreview::Text(preview)) => preview,
                _ => "".to_string(),
            },
            SignDigestType::Eip712 => {
                let value: serde_json::Value = serde_json::from_slice(&self.message.data).unwrap_or_default();
                serde_json::to_string_pretty(&value).unwrap_or_default()
            }
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        match self.message.sign_type {
            SignDigestType::Sign => self.message.data.clone(),
            SignDigestType::Eip191 => eip191_hash_message(&self.message.data).to_vec(),
            SignDigestType::Eip712 => {
                if let Ok(value) = serde_json::from_slice(&self.message.data) {
                    eip712_hash_message(value).unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            SignDigestType::Base58 => {
                if let Ok(decoded) = bs58::decode(&self.message.data).into_vec() {
                    return decoded;
                }
                Vec::new()
            }
            SignDigestType::SuiPersonalMessage => {
                let message = PersonalMessage(Cow::Borrowed(self.message.data.as_slice()));
                message.signing_digest().to_vec()
            }
        }
    }

    pub fn get_result(&self, data: &[u8]) -> String {
        match self.message.sign_type {
            SignDigestType::Eip191 | SignDigestType::Eip712 => {
                if data.len() < SIGNATURE_LENGTH {
                    return hex::encode_prefixed(data);
                }
                let mut signature = data.to_vec();
                if signature[RECOVERY_ID_INDEX] < ETHEREUM_RECOVERY_ID_OFFSET {
                    signature[RECOVERY_ID_INDEX] += ETHEREUM_RECOVERY_ID_OFFSET;
                }
                hex::encode_prefixed(&signature)
            }
            SignDigestType::Sign => hex::encode_prefixed(data),
            SignDigestType::Base58 => bs58::encode(data).into_string(),
            SignDigestType::SuiPersonalMessage => {
                if data.len() == SUI_PERSONAL_SIGNATURE_LEN {
                    BASE64.encode(data)
                } else {
                    hex::encode_prefixed(data)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{
        eip712::{GemEIP712Section, GemEIP712Value},
        sign_type::SignDigestType,
    };
    use alloy_primitives::hex;
    use gem_evm::EIP712Domain;

    #[test]
    fn test_eip191() {
        let data = b"hello world".to_vec();
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip191,
            data,
        });
        match decoder.preview() {
            Ok(MessagePreview::Text(preview)) => assert_eq!(preview, "hello world"),
            _ => panic!("Unexpected preview result"),
        }

        let hash = decoder.hash();
        assert_eq!(
            hex::encode_prefixed(&hash),
            "0xd9eba16ed0ecae432b71fe008c98cc872bb4cc214d3220a36f365326cf807d68"
        );
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
    fn test_get_result_eip191() {
        let data =
            hex::decode("d80c5ffe75fcbac0706c5c5d3b8884ae3588c30065a95075e07fa6ebc24e56433e5030992ef438b1d23437ec8d66d3197b1ad92f85222af1624d8f295907a65800")
                .expect("Invalid hex string");
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip191,
            data: data.clone(),
        });
        let result = decoder.get_result(data.as_slice());
        assert_eq!(
            result,
            "0xd80c5ffe75fcbac0706c5c5d3b8884ae3588c30065a95075e07fa6ebc24e56433e5030992ef438b1d23437ec8d66d3197b1ad92f85222af1624d8f295907a6581b"
        );
    }

    #[test]
    fn test_get_result_recovery_id_conversion() {
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip191,
            data: b"test".to_vec(),
        });

        // Test recovery ID 0 -> 27 (0x1b)
        let mut sig = vec![0u8; 65];
        sig[64] = 0;
        assert!(decoder.get_result(&sig).ends_with("1b"));

        // Test recovery ID 1 -> 28 (0x1c)
        sig[64] = 1;
        assert!(decoder.get_result(&sig).ends_with("1c"));

        // Test already converted IDs stay unchanged
        sig[64] = 27;
        assert!(decoder.get_result(&sig).ends_with("1b"));

        sig[64] = 28;
        assert!(decoder.get_result(&sig).ends_with("1c"));
    }

    #[test]
    fn test_sui_personal_message_hash() {
        let data = b"Hello, world!".to_vec();
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::SuiPersonalMessage,
            data: data.clone(),
        });

        let hash = decoder.hash();
        assert_eq!(hex::encode(hash), "b3a82fa7909fb9c9add005616e4024f8bc85a484a5623d44762db301cb2ad2d3");

        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::SuiPersonalMessage,
            data,
        });
        let mut signature = vec![0u8; 97];
        signature[0] = 0;
        signature[96] = 1;
        let expected = BASE64.encode(&signature);
        assert_eq!(decoder.get_result(&signature), expected);
    }

    #[test]
    fn test_get_result_sign_no_recovery_offset() {
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Sign,
            data: b"test".to_vec(),
        });

        let sig = vec![0u8; 65];
        let result = decoder.get_result(&sig);

        assert!(result.ends_with("00"));
        assert_eq!(
            result,
            "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_base58() {
        let message = "X3CUgCGzyn43DTAbUKnTMDzcGWMooJT2hPSZinjfN1QUgVNYYfeoJ5zg6i4Nd5coKGUrNpEYVoD";
        let data = message.as_bytes().to_vec();
        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Base58,
            data: data.clone(),
        });

        match decoder.preview() {
            Ok(MessagePreview::Text(preview)) => assert_eq!(preview, "This is an example message to be signed - 1747125759060"),
            _ => panic!("Unexpected preview result for base58"),
        }
        let hash = decoder.hash();

        assert_eq!(
            hex::encode(&hash),
            "5468697320697320616e206578616d706c65206d65737361676520746f206265207369676e6564202d2031373437313235373539303630"
        );

        let result_data = b"StV1DL6CwTryKyV"; // Data to pass to get_result, mimicking Swift test
        let result = decoder.get_result(result_data);

        assert_eq!(result, "3LRFsmWKLfsR7G5PqjytR");
    }

    #[test]
    fn test_eip712_hash() {
        let json_str = include_str!("./test/eip712_seaport.json");
        let json = serde_json::json!(json_str);
        let hash = eip712_hash_message(json).unwrap();

        assert_eq!(hex::encode(&hash), "0b8aa9f3712df0034bc29fe5b24dd88cfdba02c7f499856ab24632e2969709a8",);

        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip712,
            data: json_str.as_bytes().to_vec(),
        });
        let preview = decoder.preview().unwrap();
        assert_eq!(
            preview,
            MessagePreview::EIP712(GemEIP712Message {
                domain: EIP712Domain {
                    name: "Seaport".to_string(),
                    version: Some("1.1".to_string()),
                    chain_id: 1,
                    verifying_contract: Some("0x00000000006c3852cbEf3e08E8dF289169EdE581".to_string()),
                    salts: None,
                },
                message: vec![GemEIP712Section {
                    name: "OrderComponents".to_string(),
                    values: vec![
                        GemEIP712Value {
                            name: "offerer".to_string(),
                            value: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
                        },
                        GemEIP712Value {
                            name: "zone".to_string(),
                            value: "0x004C00500000aD104D7DBd00e3ae0A5C00560C00".to_string(),
                        },
                        GemEIP712Value {
                            name: "offer".to_string(),
                            value: "[...]".to_string(),
                        },
                        GemEIP712Value {
                            name: "startTime".to_string(),
                            value: "1658645591".to_string(),
                        },
                        GemEIP712Value {
                            name: "endTime".to_string(),
                            value: "1659250386".to_string(),
                        },
                        GemEIP712Value {
                            name: "zoneHash".to_string(),
                            value: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                        },
                        GemEIP712Value {
                            name: "salt".to_string(),
                            value: "16178208897136618".to_string(),
                        },
                        GemEIP712Value {
                            name: "conduitKey".to_string(),
                            value: "0x0000007b02230091a7ed01230072f7006a004d60a8d4e71d599b8104250f0000".to_string(),
                        },
                        GemEIP712Value {
                            name: "counter".to_string(),
                            value: "0".to_string(),
                        },
                    ],
                }],
            })
        );
    }

    #[test]
    fn test_eip712_ploymarket_hash() {
        let json_str = include_str!("./test/eip712_polymarket.json");

        let decoder = SignMessageDecoder::new(SignMessage {
            sign_type: SignDigestType::Eip712,
            data: json_str.as_bytes().to_vec(),
        });
        let preview = decoder.preview().unwrap();
        assert_eq!(
            preview,
            MessagePreview::EIP712(GemEIP712Message {
                domain: EIP712Domain {
                    name: "ClobAuthDomain".to_string(),
                    version: Some("1".to_string()),
                    chain_id: 137,
                    verifying_contract: None,
                    salts: None,
                },
                message: vec![GemEIP712Section {
                    name: "ClobAuth".to_string(),
                    values: vec![
                        GemEIP712Value {
                            name: "address".to_string(),
                            value: "0x514bcb1f9aabb904e6106bd1052b66d2706dbbb7".to_string(),
                        },
                        GemEIP712Value {
                            name: "timestamp".to_string(),
                            value: "1752326774".to_string(),
                        },
                        GemEIP712Value {
                            name: "nonce".to_string(),
                            value: "0".to_string(),
                        },
                        GemEIP712Value {
                            name: "message".to_string(),
                            value: "This message attests that I control the given wallet".to_string(),
                        },
                    ],
                }],
            })
        );
    }
}
