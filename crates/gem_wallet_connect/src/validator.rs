use std::time::{SystemTime, UNIX_EPOCH};

use crate::actions::WalletConnectTransactionType;
use crate::sign_type::SignDigestType;
use primitives::Chain;

fn current_timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

pub fn validate_sign_message(chain: Chain, sign_type: &SignDigestType, data: &str) -> Result<(), String> {
    match sign_type {
        SignDigestType::Eip712 => {
            let expected_chain_id = chain
                .network_id()
                .parse::<u64>()
                .map_err(|_| format!("Chain {} does not have a numeric network ID", chain))?;
            gem_evm::eip712::validate_eip712_chain_id(data, expected_chain_id)
        }
        SignDigestType::TonPersonal => {
            gem_ton::signer::TonSignMessageData::from_bytes(data.as_bytes()).map_err(|e| e.to_string())?;
            Ok(())
        }
        SignDigestType::Eip191 | SignDigestType::Base58 | SignDigestType::SuiPersonal | SignDigestType::Siwe | SignDigestType::BitcoinPersonal | SignDigestType::TronPersonal => {
            Ok(())
        }
    }
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
    use primitives::TransferDataOutputType;

    #[test]
    fn test_validate_eip712_chain_match() {
        assert!(validate_sign_message(Chain::Ethereum, &SignDigestType::Eip712, &mock_eip712_json(1)).is_ok());
    }

    #[test]
    fn test_validate_eip712_chain_mismatch() {
        let result = validate_sign_message(Chain::Ethereum, &SignDigestType::Eip712, &mock_eip712_json(137));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_validate_eip712_polygon() {
        assert!(validate_sign_message(Chain::Polygon, &SignDigestType::Eip712, &mock_eip712_json(137)).is_ok());
    }

    #[test]
    fn test_validate_eip191_always_ok() {
        assert!(validate_sign_message(Chain::Ethereum, &SignDigestType::Eip191, "anything").is_ok());
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
        assert!(validate_sign_message(Chain::Ton, &SignDigestType::TonPersonal, r#"{"payload":{"text":"Hello"},"domain":"example.com"}"#).is_err());

        // Invalid: unknown payload type
        assert!(validate_sign_message(Chain::Ton, &SignDigestType::TonPersonal, r#"{"payload":{"type":"unknown"},"domain":"example.com"}"#).is_err());

        // Valid: text payload
        let ton_data = TonSignMessageData::new(TonSignDataPayload::Text { text: "Hello".to_string() }, "example.com".to_string());
        assert!(validate_sign_message(Chain::Ton, &SignDigestType::TonPersonal, &String::from_utf8(ton_data.to_bytes()).unwrap()).is_ok());

        // Valid: binary payload
        let ton_data = TonSignMessageData::new(TonSignDataPayload::Binary { bytes: "SGVsbG8=".to_string() }, "example.com".to_string());
        assert!(validate_sign_message(Chain::Ton, &SignDigestType::TonPersonal, &String::from_utf8(ton_data.to_bytes()).unwrap()).is_ok());

        // Valid: cell payload
        let ton_data = TonSignMessageData::new(TonSignDataPayload::Cell { cell: "te6c".to_string() }, "example.com".to_string());
        assert!(validate_sign_message(Chain::Ton, &SignDigestType::TonPersonal, &String::from_utf8(ton_data.to_bytes()).unwrap()).is_ok());
    }
}
