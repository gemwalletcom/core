use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use primitives::Chain;
use serde_json::Value;

pub struct EthereumRequestHandler;

impl EthereumRequestHandler {
    pub fn parse_personal_sign(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let params_array = params.as_array().ok_or("Invalid params format")?;
        let data = params_array.first().and_then(|v| v.as_str()).ok_or("Missing data parameter")?.to_string();

        let data = if let Some(stripped) = data.strip_prefix("0x") {
            hex::decode(stripped)
                .map_err(|e| format!("Invalid hex data: {}", e))?
                .into_iter()
                .map(|b| format!("{:02x}", b))
                .collect()
        } else {
            data
        };

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip191,
            data,
        })
    }

    pub fn parse_sign_typed_data(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let params_array = params.as_array().ok_or("Invalid params format")?;
        let data = params_array.get(1).and_then(|v| v.as_str()).ok_or("Missing data parameter")?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip712,
            data,
        })
    }

    pub fn parse_sign_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let params_array = params.as_array().ok_or("Invalid params format")?;
        let transaction = params_array.first().ok_or("Missing transaction parameter")?;
        let data = serde_json::to_string(transaction).map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(WalletConnectAction::SignTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }

    pub fn parse_send_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let params_array = params.as_array().ok_or("Invalid params format")?;
        let transaction = params_array.first().ok_or("Missing transaction parameter")?;
        let data = serde_json::to_string(transaction).map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(WalletConnectAction::SendTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_personal_sign() {
        let params = serde_json::from_str(r#"["0x48656c6c6f"]"#).unwrap();
        let action = EthereumRequestHandler::parse_personal_sign(Chain::Ethereum, params).unwrap();
        match action {
            WalletConnectAction::SignMessage { chain, sign_type, .. } => {
                assert_eq!(chain, Chain::Ethereum);
                assert!(matches!(sign_type, SignDigestType::Eip191));
            }
            _ => panic!("Expected SignMessage action"),
        }
    }

    #[test]
    fn test_parse_sign_typed_data() {
        let params = serde_json::from_str(r#"["0x123", "{\"types\":{}}"]"#).unwrap();
        let action = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, params).unwrap();
        match action {
            WalletConnectAction::SignMessage { chain, sign_type, .. } => {
                assert_eq!(chain, Chain::Ethereum);
                assert!(matches!(sign_type, SignDigestType::Eip712));
            }
            _ => panic!("Expected SignMessage action"),
        }
    }

    #[test]
    fn test_parse_send_transaction() {
        let params = serde_json::from_str(r#"[{"to":"0x123","value":"0x0"}]"#).unwrap();
        let action = EthereumRequestHandler::parse_send_transaction(Chain::Ethereum, params).unwrap();
        match action {
            WalletConnectAction::SendTransaction { chain, transaction_type, .. } => {
                assert_eq!(chain, Chain::Ethereum);
                assert!(matches!(transaction_type, WalletConnectTransactionType::Ethereum));
            }
            _ => panic!("Expected SendTransaction action"),
        }
    }

    #[test]
    fn test_parse_sign_typed_data_full() {
        let params = serde_json::from_str(r#"["0x1234567890abcdef1234567890abcdef12345678", "{\"types\":{\"EIP712Domain\":[]}}"]"#).unwrap();
        let action = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, params).unwrap();
        match action {
            WalletConnectAction::SignMessage { chain, sign_type, data } => {
                assert_eq!(chain, Chain::Ethereum);
                assert!(matches!(sign_type, SignDigestType::Eip712));
                assert_eq!(data, r#"{"types":{"EIP712Domain":[]}}"#);
            }
            _ => panic!("Expected SignMessage action"),
        }
    }
}
