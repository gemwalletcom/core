use crate::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use gem_ton::signer::TonSignMessageData;
use primitives::{Chain, TransferDataOutputType, ValueAccess, WCTonSendTransaction};
use serde_json::Value;

pub struct TonRequestHandler;

fn extract_host(url: &str) -> String {
    url::Url::parse(url).map(|u| u.host_str().unwrap_or(url).to_string()).unwrap_or_else(|_| url.to_string())
}

impl TonRequestHandler {
    pub fn parse_sign_message(_chain: Chain, params: Value, domain: &str) -> Result<WalletConnectAction, String> {
        let payload = params.at(0)?.clone();
        let from = payload.get("from").and_then(|value| value.as_str()).map(|value| value.to_string());
        let host = extract_host(domain);
        let ton_data = TonSignMessageData::from_value(payload, host, from).map_err(|e| e.to_string())?;
        let data = String::from_utf8(ton_data.to_bytes()).map_err(|e| format!("Failed to encode TonSignMessageData: {}", e))?;
        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Ton,
            sign_type: SignDigestType::TonPersonal,
            data,
        })
    }

    pub fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        serde_json::from_value::<WCTonSendTransaction>(params.clone()).map_err(|e| e.to_string())?;
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
    use gem_ton::signer::TonSignDataPayload;

    #[test]
    fn test_parse_sign_message() {
        let params = serde_json::from_str(r#"[{"type":"text","text":"Hello TON","from":"UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg"}]"#).unwrap();
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params, "https://react-app.walletconnect.com").unwrap();
        let WalletConnectAction::SignMessage { chain, sign_type, data } = action else {
            panic!("Expected SignMessage action")
        };
        assert_eq!(chain, Chain::Ton);
        assert_eq!(sign_type, SignDigestType::TonPersonal);

        let parsed: TonSignMessageData = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.domain, "react-app.walletconnect.com");
        assert_eq!(parsed.address.as_deref(), Some("UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg"));
        assert_eq!(parsed.payload, TonSignDataPayload::Text { text: "Hello TON".to_string() });
    }

    #[test]
    fn test_parse_sign_message_extracts_host() {
        let params = serde_json::from_str(r#"[{"type":"text","text":"Test","from":"UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg"}]"#).unwrap();
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params, "https://example.com/path?query=1").unwrap();
        let WalletConnectAction::SignMessage { data, .. } = action else {
            panic!("Expected SignMessage action")
        };

        let parsed: TonSignMessageData = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.domain, "example.com");
    }

    #[test]
    fn test_parse_sign_message_without_from() {
        let params = serde_json::from_str(r#"[{"type":"text","text":"Hello TON"}]"#).unwrap();
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params, "https://react-app.walletconnect.com").unwrap();
        let WalletConnectAction::SignMessage { data, .. } = action else {
            panic!("Expected SignMessage action")
        };

        let parsed: TonSignMessageData = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.domain, "react-app.walletconnect.com");
        assert_eq!(parsed.address, None);
        assert_eq!(parsed.payload, TonSignDataPayload::Text { text: "Hello TON".to_string() });
    }

    #[test]
    fn test_parse_sign_message_preserves_cell_schema() {
        let params = serde_json::from_str(
            r#"[{"type":"cell","schema":"comment#00000000 text:SnakeData = InMsgBody;","cell":"te6c","from":"UQBY1cVPu4SIr36q0M3HWcqPb_efyVVRBsEzmwN-wKQDR6zg","network":"-239"}]"#,
        )
        .unwrap();
        let action = TonRequestHandler::parse_sign_message(Chain::Ton, params, "https://react-app.walletconnect.com").unwrap();
        let WalletConnectAction::SignMessage { data, .. } = action else {
            panic!("Expected SignMessage action")
        };

        let parsed: TonSignMessageData = serde_json::from_str(&data).unwrap();
        assert_eq!(
            parsed.payload,
            TonSignDataPayload::Cell {
                schema: "comment#00000000 text:SnakeData = InMsgBody;".to_string(),
                cell: "te6c".to_string(),
            }
        );
        assert_eq!(parsed.network.as_deref(), Some("-239"));
    }

    #[test]
    fn test_parse_send_transaction() {
        let params: Value = serde_json::from_str(r#"{"valid_until":1234567890,"messages":[{"address":"0:1234567890abcdef","amount":"1000000000"}]}"#).unwrap();
        let action = TonRequestHandler::parse_send_transaction(Chain::Ton, params).unwrap();

        let WalletConnectAction::SendTransaction { chain, transaction_type, .. } = action else {
            panic!("Expected SendTransaction action")
        };
        assert_eq!(chain, Chain::Ton);
        assert_eq!(transaction_type.get_output_type().unwrap(), TransferDataOutputType::EncodedTransaction);
    }

    #[test]
    fn test_parse_send_transaction_missing_messages() {
        let params = serde_json::from_str(r#"{"valid_until": 123}"#).unwrap();
        assert!(TonRequestHandler::parse_send_transaction(Chain::Ton, params).is_err());
    }
}
