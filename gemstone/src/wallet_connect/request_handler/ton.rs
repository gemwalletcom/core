use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType};
use serde_json::Value;
use signer::TonSignMessageData;

pub struct TonRequestHandler;

fn extract_host(url: &str) -> String {
    url::Url::parse(url).map(|u| u.host_str().unwrap_or(url).to_string()).unwrap_or_else(|_| url.to_string())
}

impl ChainRequestHandler for TonRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value, domain: &str) -> Result<WalletConnectAction, String> {
        let params_array = params.as_array().ok_or("Invalid params format")?;
        let payload = params_array.first().ok_or("Missing payload parameter")?.clone();
        let host = extract_host(domain);
        let ton_data = TonSignMessageData::new(payload, host);
        let data = String::from_utf8(ton_data.to_bytes()).map_err(|e| format!("Failed to encode TonSignMessageData: {}", e))?;
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
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params, "https://react-app.walletconnect.com").unwrap();
        let WalletConnectAction::SignMessage { chain, sign_type, data } = action else {
            panic!("Expected SignMessage action")
        };
        assert_eq!(chain, Chain::Ton);
        assert_eq!(sign_type, SignDigestType::TonPersonal);

        let parsed: TonSignMessageData = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.domain, "react-app.walletconnect.com");
        assert_eq!(parsed.payload["type"], "text");
        assert_eq!(parsed.payload["text"], "Hello TON");
    }

    #[test]
    fn test_parse_sign_message_extracts_host() {
        let params = serde_json::from_str(r#"[{"type":"text","text":"Test"}]"#).unwrap();
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params, "https://example.com/path?query=1").unwrap();
        let WalletConnectAction::SignMessage { data, .. } = action else {
            panic!("Expected SignMessage action")
        };

        let parsed: TonSignMessageData = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.domain, "example.com");
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
        assert_eq!(parsed_data["valid_until"], 1234567890);
        assert_eq!(parsed_data["messages"][0]["address"], "0:1234567890abcdef");
        assert_eq!(parsed_data["messages"][0]["amount"], "1000000000");
    }

    #[test]
    fn test_parse_send_transaction_missing_messages() {
        let params = serde_json::from_str(r#"{"valid_until": 123}"#).unwrap();
        let result = TonRequestHandler::parse_send_transaction(Chain::Ton, params);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing messages parameter");
    }
}
