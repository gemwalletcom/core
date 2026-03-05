use crate::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::sign_type::SignDigestType;
use primitives::{Chain, ValueAccess};
use serde_json::Value;

pub struct EthereumRequestHandler;

impl EthereumRequestHandler {
    pub fn parse_sign_message(chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let data = params.at(0)?.string()?.to_string();

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip191,
            data,
        })
    }

    pub fn parse_sign_typed_data(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let typed_data = params.at(1)?;
        let data = if let Some(s) = typed_data.as_str() {
            s.to_string()
        } else {
            serde_json::to_string(typed_data).map_err(|e| format!("Failed to serialize typed data: {}", e))?
        };

        let expected_chain_id = chain
            .network_id()
            .parse::<u64>()
            .map_err(|_| format!("Chain {} does not have a numeric network ID", chain))?;
        gem_evm::eip712::validate_eip712_chain_id(&data, expected_chain_id)?;

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::Eip712,
            data,
        })
    }

    pub fn parse_sign_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params.at(0)?;
        let data = serde_json::to_string(transaction).map_err(|e| format!("Failed to serialize transaction: {}", e))?;

        Ok(WalletConnectAction::SignTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Ethereum,
            data,
        })
    }

    pub fn parse_send_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transaction = params.at(0)?;
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
    use gem_evm::testkit::eip712_mock::mock_eip712_json;

    fn eip712_params(chain_id: u64) -> Value {
        let eip712_json = mock_eip712_json(chain_id);
        serde_json::json!(["0x123", eip712_json])
    }

    fn eip712_params_object(chain_id: u64) -> Value {
        let eip712_value: Value = serde_json::from_str(&mock_eip712_json(chain_id)).unwrap();
        serde_json::json!(["0x123", eip712_value])
    }

    #[test]
    fn test_parse_personal_sign() {
        let params = serde_json::from_str(r#"["0x48656c6c6f"]"#).unwrap();
        let action = EthereumRequestHandler::parse_sign_message(Chain::Ethereum, params, "example.com").unwrap();
        assert_eq!(
            action,
            WalletConnectAction::SignMessage {
                chain: Chain::Ethereum,
                sign_type: SignDigestType::Eip191,
                data: "0x48656c6c6f".to_string(),
            }
        );
    }

    #[test]
    fn test_parse_sign_typed_data_matching_chain() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params(1));
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_chain_id_mismatch_polygon_on_ethereum() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params(137));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_sign_typed_data_chain_id_mismatch_ethereum_on_polygon() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Polygon, eip712_params(1));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
    }

    #[test]
    fn test_parse_sign_typed_data_polygon_matching() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::Polygon, eip712_params(137)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_bsc_matching() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::SmartChain, eip712_params(56)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_arbitrum_matching() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::Arbitrum, eip712_params(42161)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_object_params() {
        assert!(EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params_object(1)).is_ok());
    }

    #[test]
    fn test_parse_sign_typed_data_object_params_chain_mismatch() {
        let result = EthereumRequestHandler::parse_sign_typed_data(Chain::Ethereum, eip712_params_object(137));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain ID mismatch"));
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
}
