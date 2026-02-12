use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use primitives::{Chain, ValueAccess};
use serde_json::Value;

pub struct EthereumRequestHandler;

impl ChainRequestHandler for EthereumRequestHandler {
    fn parse_sign_message(chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let data = params.at(0)?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip191,
            data,
        })
    }

    fn parse_sign_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params.at(0)?;
        let data = serde_json::to_string(transaction).map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(WalletConnectAction::SignTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }

    fn parse_send_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params.at(0)?;
        let data = serde_json::to_string(transaction).map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(WalletConnectAction::SendTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }
}

impl EthereumRequestHandler {
    pub fn parse_sign_typed_data(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let typed_data = params.at(1)?;
        let data = if let Some(s) = typed_data.as_str() {
            s.to_string()
        } else {
            serde_json::to_string(typed_data).map_err(|e| format!("Failed to serialize typed data: {}", e))?
        };

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip712,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet_connect::handler_traits::ChainRequestHandler;

    fn sign_message(chain: Chain, sign_type: SignDigestType, data: &str) -> WalletConnectAction {
        WalletConnectAction::SignMessage {
            chain,
            sign_type,
            data: data.to_string(),
        }
    }

    #[test]
    fn test_parse_personal_sign() {
        let params = serde_json::from_str(r#"["0x48656c6c6f"]"#).unwrap();
        assert_eq!(
            EthereumRequestHandler::parse_sign_message(Chain::Ethereum, params, "example.com").unwrap(),
            sign_message(Chain::Ethereum, SignDigestType::Eip191, "0x48656c6c6f")
        );
    }

    #[test]
    fn test_parse_sign_typed_data() {
        let params = serde_json::from_str(r#"["0x123", "{\"types\":{}}"]"#).unwrap();
        assert_eq!(
            EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, params).unwrap(),
            sign_message(Chain::Ethereum, SignDigestType::Eip712, r#"{"types":{}}"#)
        );
    }

    #[test]
    fn test_parse_sign_typed_data_hyperliquid() {
        let params = serde_json::from_str(r#"["0x123", {"types":{}}]"#).unwrap();
        assert_eq!(
            EthereumRequestHandler::parse_sign_typed_data(Chain::Arbitrum, params).unwrap(),
            sign_message(Chain::Arbitrum, SignDigestType::Eip712, r#"{"types":{}}"#)
        );
    }

    #[test]
    fn test_parse_send_transaction() {
        let params = serde_json::from_str(r#"[{"to":"0x123","value":"0x0"}]"#).unwrap();
        assert_eq!(
            EthereumRequestHandler::parse_send_transaction(Chain::Ethereum, params).unwrap(),
            WalletConnectAction::SendTransaction {
                chain: Chain::Ethereum,
                transaction_type: WalletConnectTransactionType::Ethereum,
                data: r#"{"to":"0x123","value":"0x0"}"#.to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sign_typed_data_full() {
        let params = serde_json::from_str(r#"["0x1234567890abcdef1234567890abcdef12345678", "{\"types\":{\"EIP712Domain\":[]}}"]"#).unwrap();
        assert_eq!(
            EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, params).unwrap(),
            sign_message(Chain::Ethereum, SignDigestType::Eip712, r#"{"types":{"EIP712Domain":[]}}"#)
        );
    }

    #[test]
    fn test_parse_personal_sign_ignores_siwe_detection() {
        let message = [
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
        .join("\n");
        let params = serde_json::json!([message.clone()]);
        assert_eq!(
            EthereumRequestHandler::parse_sign_message(Chain::Ethereum, params, "example.com").unwrap(),
            sign_message(Chain::Ethereum, SignDigestType::Eip191, &message)
        );
    }
}
