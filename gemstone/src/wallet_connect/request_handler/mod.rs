mod ethereum;
mod solana;
mod sui;

use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectChainOperation};
use ethereum::EthereumRequestHandler;
use primitives::{Chain, WalletConnectRequest, WalletConnectionMethods};
use serde_json::Value;
use solana::SolanaRequestHandler;
use sui::SuiRequestHandler;

pub struct WalletConnectRequestHandler;

impl WalletConnectRequestHandler {
    pub fn parse_request(request: WalletConnectRequest) -> Result<WalletConnectAction, String> {
        let method = serde_json::from_value::<WalletConnectionMethods>(serde_json::Value::String(request.method.clone()))
            .map_err(|_| format!("Unsupported method: {}", request.method))?;
        let params = serde_json::from_str::<Value>(&request.params).map_err(|e| format!("Failed to parse params: {}", e))?;

        match method {
            WalletConnectionMethods::PersonalSign => {
                let chain = Self::resolve_chain(request.chain_id)?;
                EthereumRequestHandler::parse_personal_sign(chain, params)
            }
            WalletConnectionMethods::EthSignTypedData | WalletConnectionMethods::EthSignTypedDataV4 => {
                let chain = Self::resolve_chain(request.chain_id)?;
                EthereumRequestHandler::parse_sign_typed_data(chain, params)
            }
            WalletConnectionMethods::EthSignTransaction => {
                let chain = Self::resolve_chain(request.chain_id)?;
                EthereumRequestHandler::parse_sign_transaction(chain, params)
            }
            WalletConnectionMethods::EthSendTransaction => {
                let chain = Self::resolve_chain(request.chain_id)?;
                EthereumRequestHandler::parse_send_transaction(chain, params)
            }
            WalletConnectionMethods::EthSendRawTransaction => Err("Method not supported".to_string()),
            WalletConnectionMethods::EthChainId => Ok(WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::GetChainId,
            }),
            WalletConnectionMethods::WalletAddEthereumChain => Ok(WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::AddChain,
            }),
            WalletConnectionMethods::WalletSwitchEthereumChain => Ok(WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::SwitchChain,
            }),
            WalletConnectionMethods::SolanaSignMessage => SolanaRequestHandler::parse_sign_message(params),
            WalletConnectionMethods::SolanaSignTransaction => SolanaRequestHandler::parse_sign_transaction(params),
            WalletConnectionMethods::SolanaSignAndSendTransaction => SolanaRequestHandler::parse_send_transaction(params),
            WalletConnectionMethods::SolanaSignAllTransactions => SolanaRequestHandler::parse_sign_all_transactions(params),
            WalletConnectionMethods::SuiSignPersonalMessage => SuiRequestHandler::parse_sign_message(params),
            WalletConnectionMethods::SuiSignTransaction => SuiRequestHandler::parse_sign_transaction(params),
            WalletConnectionMethods::SuiSignAndExecuteTransaction => SuiRequestHandler::parse_send_transaction(params),
        }
    }

    fn resolve_chain(chain_id: Option<String>) -> Result<Chain, String> {
        primitives::WalletConnectCAIP2::resolve_chain(chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_method() {
        let request = WalletConnectRequest {
            topic: "test-topic".to_string(),
            method: "unknown_method".to_string(),
            params: "{}".to_string(),
            chain_id: None,
        };

        let result = WalletConnectRequestHandler::parse_request(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_operation_add_chain() {
        let request = WalletConnectRequest {
            topic: "test-topic".to_string(),
            method: "wallet_addEthereumChain".to_string(),
            params: "{}".to_string(),
            chain_id: None,
        };

        let action = WalletConnectRequestHandler::parse_request(request).unwrap();
        match action {
            WalletConnectAction::ChainOperation { operation } => {
                assert!(matches!(operation, WalletConnectChainOperation::AddChain));
            }
            _ => panic!("Expected ChainOperation action"),
        }
    }
}
