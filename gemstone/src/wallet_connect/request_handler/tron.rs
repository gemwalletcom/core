use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::error::ValueExt;
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, TransferDataOutputType};
use serde_json::Value;

// https://docs.reown.com/advanced/multichain/rpc-reference/tron-rpc
pub struct TronRequestHandler;

impl ChainRequestHandler for TronRequestHandler {
    fn parse_sign_message(_chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params.get_str("message")?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain: Chain::Tron,
            sign_type: SignDigestType::TronPersonal,
            data: message,
        })
    }

    fn parse_sign_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_param("transaction")?;

        Ok(WalletConnectAction::SignTransaction {
            chain: Chain::Tron,
            transaction_type: WalletConnectTransactionType::Tron {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data: params.to_string(),
        })
    }

    fn parse_send_transaction(_chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        params.get_param("transaction")?;

        Ok(WalletConnectAction::SendTransaction {
            chain: Chain::Tron,
            transaction_type: WalletConnectTransactionType::Tron {
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
        let params = serde_json::from_str(r#"{"message":"Hello"}"#).unwrap();
        let action = TronRequestHandler::parse_sign_message(Chain::Tron, params, "example.com").unwrap();
        let WalletConnectAction::SignMessage { chain, sign_type, data } = action else {
            panic!("Expected SignMessage action")
        };
        assert_eq!(chain, Chain::Tron);
        assert_eq!(sign_type, SignDigestType::TronPersonal);
        assert_eq!(data, "Hello");
    }

    #[test]
    fn test_parse_sign_transaction() {
        let params = serde_json::from_str(r#"{"transaction":{"raw_data_hex":"abc"}}"#).unwrap();
        let action = TronRequestHandler::parse_sign_transaction(Chain::Tron, params).unwrap();
        let WalletConnectAction::SignTransaction { chain, transaction_type, data } = action else {
            panic!("Expected SignTransaction action")
        };
        assert_eq!(chain, Chain::Tron);
        assert_eq!(transaction_type.get_output_type().unwrap(), TransferDataOutputType::EncodedTransaction);
        let parsed_data: serde_json::Value = serde_json::from_str(&data).unwrap();
        assert!(parsed_data.get("transaction").is_some());
    }

    #[test]
    fn test_parse_send_transaction() {
        let params = serde_json::from_str(r#"{"transaction":{"raw_data_hex":"abc"}}"#).unwrap();
        let action = TronRequestHandler::parse_send_transaction(Chain::Tron, params).unwrap();
        let WalletConnectAction::SendTransaction { chain, transaction_type, data } = action else {
            panic!("Expected SendTransaction action")
        };
        assert_eq!(chain, Chain::Tron);
        assert_eq!(transaction_type.get_output_type().unwrap(), TransferDataOutputType::EncodedTransaction);
        let parsed_data: serde_json::Value = serde_json::from_str(&data).unwrap();
        assert!(parsed_data.get("transaction").is_some());
    }
}
