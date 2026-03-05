use gem_evm::siwe::SiweMessage;
use hex::FromHex;
use primitives::Chain;

use crate::sign_type::{SignDigestType, SignMessage};

pub fn decode_sign_message(chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
    let mut utf8_value = None;
    let message_data = if let Some(stripped) = data.strip_prefix("0x") {
        Vec::from_hex(stripped).unwrap_or_else(|_| data.as_bytes().to_vec())
    } else {
        utf8_value = Some(data.clone());
        data.into_bytes()
    };

    let raw_text = utf8_value.or_else(|| String::from_utf8(message_data.clone()).ok()).unwrap_or_default();

    if sign_type == SignDigestType::Eip191
        && let Some(siwe_message) = decode_siwe_message(chain, &raw_text, &message_data)
    {
        return siwe_message;
    }

    SignMessage {
        chain,
        sign_type,
        data: message_data,
    }
}

fn decode_siwe_message(chain: Chain, raw_text: &str, message_data: &[u8]) -> Option<SignMessage> {
    let message = SiweMessage::try_parse(raw_text)?;
    message.validate(chain).ok()?;

    Some(SignMessage {
        chain,
        sign_type: SignDigestType::Siwe,
        data: message_data.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_siwe_message() -> String {
        [
            "login.xyz wants you to sign in with your Ethereum account:",
            "0x6dD7802E6d44bE89a789C4bD60bD511B68F41c7c",
            "",
            "Sign in with Ethereum to the app.",
            "",
            "URI: https://login.xyz",
            "Version: 1",
            "Chain ID: 1",
            "Nonce: 8hK9pX32",
            "Issued At: 2024-04-01T12:00:00Z",
        ]
        .join("\n")
    }

    #[test]
    fn test_decode_sign_message_detects_siwe() {
        let message = sample_siwe_message();
        let decoded = decode_sign_message(Chain::Ethereum, SignDigestType::Eip191, message.clone());

        assert_eq!(decoded.sign_type, SignDigestType::Siwe);
        assert_eq!(decoded.data, message.into_bytes());
    }

    #[test]
    fn test_decode_sign_message_preserves_non_siwe() {
        let message = "Hello world".to_string();
        let decoded = decode_sign_message(Chain::Ethereum, SignDigestType::Eip191, message.clone());

        assert_eq!(decoded.sign_type, SignDigestType::Eip191);
        assert_eq!(decoded.data, message.into_bytes());
    }

    #[test]
    fn test_decode_sign_message_siwe_chain_mismatch() {
        let message = sample_siwe_message();
        let decoded = decode_sign_message(Chain::Polygon, SignDigestType::Eip191, message);

        assert_eq!(decoded.sign_type, SignDigestType::Eip191);
    }
}
