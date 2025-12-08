use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType};
use serde_json::Value;

pub struct TonRequestHandler;

impl ChainRequestHandler for TonRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let params_array = params.as_array().ok_or("Invalid params format")?;
        let payload = params_array.first().ok_or("Missing payload parameter")?;
        let data = payload.to_string();
        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Ton,
            sign_type: SignDigestType::TonPersonal,
            data,
        })
    }

    fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get("messages").ok_or("Missing messages parameter")?;
        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Ton,
            transaction_type: WalletConnectTransactionType::Ton {
                output_type: TransferDataOutputType::Signature,
            },
            data: params.to_string(),
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get("messages").ok_or("Missing messages parameter")?;
        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Ton,
            transaction_type: WalletConnectTransactionType::Ton {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sign_message() {
        let params = serde_json::from_str(r#"[{"type":"text","text":"Hello TON"}]"#).unwrap();
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params).unwrap();
        let WalletConnectAction::SignMessage { chain, sign_type, data } = action else {
            panic!("Expected SignMessage action")
        };
        assert_eq!(chain, Chain::Ton);
        assert_eq!(sign_type, SignDigestType::TonPersonal);
        assert_eq!(data, r#"{"type":"text","text":"Hello TON"}"#);
    }

    #[test]
    fn test_parse_send_transaction() {
        let params_json = r#"{
            "valid_until": 1234567890,
            "messages": [
                {
                    "address": "0:1234567890abcdef",
                    "amount": "1000000000"
                }
            ]
        }"#;
        let params = serde_json::from_str(params_json).unwrap();
        let action = TonRequestHandler::parse_send_transaction(Chain::Ton, params).unwrap();

        let WalletConnectAction::SendTransaction { chain, transaction_type, data } = action else {
            panic!("Expected SendTransaction action")
        };
        assert_eq!(chain, Chain::Ton);
        let WalletConnectTransactionType::Ton {
            output_type: TransferDataOutputType::EncodedTransaction,
        } = transaction_type
        else {
            panic!("Expected Ton transaction type with EncodedTransaction output")
        };
        let parsed_data: serde_json::Value = serde_json::from_str(&data).expect("Data should be valid JSON");
        assert!(parsed_data.get("messages").is_some());
    }

    #[test]
    fn test_parse_send_transaction_missing_messages() {
        let params = serde_json::from_str(r#"{"valid_until": 123}"#).unwrap();
        let result = TonRequestHandler::parse_send_transaction(Chain::Ton, params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing messages parameter");
    }
}
