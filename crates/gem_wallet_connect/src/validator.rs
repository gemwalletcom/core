use std::time::{SystemTime, UNIX_EPOCH};

use crate::actions::WalletConnectTransactionType;
use crate::sign_type::SignDigestType;
use gem_evm::domain::host_only;
use gem_evm::siwe::SiweMessage;
use primitives::Chain;

pub struct SignMessageValidation<'a> {
    pub chain: Chain,
    pub sign_type: &'a SignDigestType,
    pub data: &'a str,
    pub session_domain: &'a str,
}

fn current_timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub fn validate_sign_message(input: &SignMessageValidation) -> Result<(), String> {
    match input.sign_type {
        SignDigestType::Eip712 => {
            let expected_chain_id = input
                .chain
                .network_id()
                .parse::<u64>()
                .map_err(|_| format!("Chain {} does not have a numeric network ID", input.chain))?;
            gem_evm::eip712::validate_eip712_chain_id(input.data, expected_chain_id)
        }
        SignDigestType::TonPersonal => {
            gem_ton::signer::TonSignMessageData::from_bytes(input.data.as_bytes()).map_err(|e| e.to_string())?;
            Ok(())
        }
        SignDigestType::Eip191 | SignDigestType::Siwe => validate_siwe(input),
        SignDigestType::Base58 | SignDigestType::SuiPersonal | SignDigestType::BitcoinPersonal | SignDigestType::TronPersonal => Ok(()),
    }
}

fn validate_siwe(input: &SignMessageValidation) -> Result<(), String> {
    let text = decode_text(input.data);
    let Some(message) = text.as_deref().and_then(SiweMessage::try_parse) else {
        if *input.sign_type == SignDigestType::Siwe {
            return Err("Invalid SIWE message".to_string());
        }
        return Ok(());
    };
    message.validate(input.chain)?;
    validate_session_domain(&message, input.session_domain)
}

fn decode_text(data: &str) -> Option<String> {
    if let Some(stripped) = data.strip_prefix("0x") {
        hex::decode(stripped).ok().and_then(|bytes| String::from_utf8(bytes).ok())
    } else {
        Some(data.to_string())
    }
}

fn validate_session_domain(message: &SiweMessage, session_domain: &str) -> Result<(), String> {
    let session_host = host_only(session_domain).ok_or_else(|| "Invalid session origin".to_string())?;
    let message_host = host_only(&message.domain).ok_or_else(|| "Invalid SIWE domain".to_string())?;
    if session_host != message_host {
        return Err(format!("Domain mismatch: SIWE domain {} does not match session origin {}", message.domain, session_domain));
    }
    Ok(())
}

pub fn validate_send_transaction(transaction_type: &WalletConnectTransactionType, data: &str) -> Result<(), String> {
    let WalletConnectTransactionType::Ton { .. } = transaction_type else {
        return Ok(());
    };

    let json: serde_json::Value = serde_json::from_str(data).map_err(|_| "Invalid JSON".to_string())?;

    if let Some(valid_until) = json.get("valid_until").and_then(|v| v.as_i64())
        && current_timestamp() >= valid_until
    {
        return Err("Transaction expired".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_evm::testkit::eip712_mock::mock_eip712_json;
    use gem_evm::testkit::siwe_mock::{mock_siwe_message, mock_siwe_message_hex};
    use primitives::TransferDataOutputType;

    fn sign_validation<'a>(chain: Chain, sign_type: &'a SignDigestType, data: &'a str, session_domain: &'a str) -> SignMessageValidation<'a> {
        SignMessageValidation {
            chain,
            sign_type,
            data,
            session_domain,
        }
    }

    #[test]
    fn test_validate_eip712_chain_match() {
        assert!(validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Eip712, &mock_eip712_json(1), "")).is_ok());
    }

    #[test]
    fn test_validate_eip712_chain_mismatch() {
        let result = validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Eip712, &mock_eip712_json(137), ""));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_validate_eip712_polygon() {
        assert!(validate_sign_message(&sign_validation(Chain::Polygon, &SignDigestType::Eip712, &mock_eip712_json(137), "")).is_ok());
    }

    #[test]
    fn test_validate_eip191_always_ok() {
        assert!(validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Eip191, "anything", "example.com")).is_ok());
    }

    #[test]
    fn test_validate_ton_send_transaction_expired() {
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(validate_send_transaction(&ton_type, r#"{"valid_until": 1234567890, "messages": []}"#).is_err());
    }

    #[test]
    fn test_validate_ton_send_transaction_valid() {
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(validate_send_transaction(&ton_type, r#"{"valid_until": 9999999999, "messages": []}"#).is_ok());
    }

    #[test]
    fn test_validate_ethereum_send_transaction_always_ok() {
        assert!(validate_send_transaction(&WalletConnectTransactionType::Ethereum, "{}").is_ok());
    }

    #[test]
    fn test_validate_ton_send_transaction_no_expiry() {
        let ton_type = WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        };
        assert!(validate_send_transaction(&ton_type, r#"{"messages": []}"#).is_ok());
    }

    #[test]
    fn test_validate_ton_sign_message() {
        use gem_ton::signer::{TonSignDataPayload, TonSignMessageData};

        // Invalid: raw JSON without proper encoding
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ton,
                &SignDigestType::TonPersonal,
                r#"{"payload":{"text":"Hello"},"domain":"example.com"}"#,
                ""
            ))
            .is_err()
        );

        // Invalid: unknown payload type
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ton,
                &SignDigestType::TonPersonal,
                r#"{"payload":{"type":"unknown"},"domain":"example.com"}"#,
                ""
            ))
            .is_err()
        );

        // Valid: text payload
        let ton_data = TonSignMessageData::new(
            TonSignDataPayload::Text { text: "Hello".to_string() },
            "example.com".to_string(),
            "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
        );
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ton,
                &SignDigestType::TonPersonal,
                &String::from_utf8(ton_data.to_bytes()).unwrap(),
                ""
            ))
            .is_ok()
        );

        // Valid: binary payload
        let ton_data = TonSignMessageData::new(
            TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() },
            "example.com".to_string(),
            "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
        );
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ton,
                &SignDigestType::TonPersonal,
                &String::from_utf8(ton_data.to_bytes()).unwrap(),
                ""
            ))
            .is_ok()
        );

        // Valid: cell payload
        let ton_data = TonSignMessageData::new(
            TonSignDataPayload::Cell { cell: "te6c".to_string() },
            "example.com".to_string(),
            "UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg".to_string(),
        );
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ton,
                &SignDigestType::TonPersonal,
                &String::from_utf8(ton_data.to_bytes()).unwrap(),
                ""
            ))
            .is_ok()
        );
    }

    #[test]
    fn test_validate_siwe() {
        let valid = mock_siwe_message("thepoc.xyz", 1);
        assert!(validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Siwe, &valid, "https://thepoc.xyz")).is_ok());

        let with_port = mock_siwe_message("thepoc.xyz:8080", 1);
        assert!(validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Siwe, &with_port, "https://thepoc.xyz")).is_ok());

        let chain_mismatch = mock_siwe_message("thepoc.xyz", 137);
        assert!(
            validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Siwe, &chain_mismatch, "https://thepoc.xyz"))
                .unwrap_err()
                .contains("Chain ID mismatch")
        );

        let domain_mismatch = mock_siwe_message("evil.com", 1);
        assert!(
            validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Siwe, &domain_mismatch, "https://thepoc.xyz"))
                .unwrap_err()
                .contains("Domain mismatch")
        );

        assert!(
            validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Siwe, "not siwe", "https://thepoc.xyz"))
                .unwrap_err()
                .contains("Invalid SIWE message")
        );
    }

    #[test]
    fn test_validate_eip191_siwe() {
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ethereum,
                &SignDigestType::Eip191,
                &mock_siwe_message_hex("thepoc.xyz", 137),
                "https://thepoc.xyz"
            ))
            .unwrap_err()
            .contains("Chain ID mismatch")
        );
        assert!(
            validate_sign_message(&sign_validation(
                Chain::Ethereum,
                &SignDigestType::Eip191,
                &mock_siwe_message_hex("evil.com", 1),
                "https://thepoc.xyz"
            ))
            .unwrap_err()
            .contains("Domain mismatch")
        );
        assert!(validate_sign_message(&sign_validation(Chain::Ethereum, &SignDigestType::Eip191, "0x48656c6c6f", "https://example.com")).is_ok());
    }
}
