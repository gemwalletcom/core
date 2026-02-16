mod bitcoin;
mod ethereum;
mod solana;
mod sui;
mod ton;
mod tron;

use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectChainOperation};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use bitcoin::BitcoinRequestHandler;
use ethereum::EthereumRequestHandler;
use primitives::{Chain, WalletConnectRequest, WalletConnectionMethods};
use serde_json::Value;
use solana::SolanaRequestHandler;
use sui::SuiRequestHandler;
use ton::TonRequestHandler;
use tron::TronRequestHandler;

pub struct WalletConnectRequestHandler;

impl WalletConnectRequestHandler {
    pub fn parse_request(request: WalletConnectRequest) -> Result<WalletConnectAction, String> {
        let method =
            serde_json::from_value::<WalletConnectionMethods>(serde_json::Value::String(request.method.clone())).map_err(|_| format!("Unsupported method: {}", request.method))?;
        let params = serde_json::from_str::<Value>(&request.params).map_err(|e| format!("Failed to parse params: {}", e))?;
        let params = match params {
            Value::String(raw_json) => serde_json::from_str::<Value>(&raw_json).unwrap_or(Value::String(raw_json)),
            value => value,
        };

        let domain = &request.domain;

        match method {
            WalletConnectionMethods::PersonalSign => {
                let chain = Self::resolve_chain(request.chain_id)?;
                EthereumRequestHandler::parse_sign_message(chain, params, domain)
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
            WalletConnectionMethods::SolanaSignMessage => SolanaRequestHandler::parse_sign_message(Chain::Solana, params, domain),
            WalletConnectionMethods::SolanaSignTransaction => SolanaRequestHandler::parse_sign_transaction(Chain::Solana, params),
            WalletConnectionMethods::SolanaSignAndSendTransaction => SolanaRequestHandler::parse_send_transaction(Chain::Solana, params),
            WalletConnectionMethods::SolanaSignAllTransactions => SolanaRequestHandler::parse_sign_all_transactions(params),
            WalletConnectionMethods::SuiSignPersonalMessage => SuiRequestHandler::parse_sign_message(Chain::Sui, params, domain),
            WalletConnectionMethods::SuiSignTransaction => SuiRequestHandler::parse_sign_transaction(Chain::Sui, params),
            WalletConnectionMethods::SuiSignAndExecuteTransaction => SuiRequestHandler::parse_send_transaction(Chain::Sui, params),
            WalletConnectionMethods::TonSignData => TonRequestHandler::parse_sign_message(Chain::Ton, params, domain),
            WalletConnectionMethods::TonSendMessage => TonRequestHandler::parse_send_transaction(Chain::Ton, params),
            WalletConnectionMethods::TronSignMessage => TronRequestHandler::parse_sign_message(Chain::Tron, params, domain),
            WalletConnectionMethods::TronSignTransaction => TronRequestHandler::parse_sign_transaction(Chain::Tron, params),
            WalletConnectionMethods::TronSendTransaction => TronRequestHandler::parse_send_transaction(Chain::Tron, params),
            WalletConnectionMethods::BtcSignMessage => BitcoinRequestHandler::parse_sign_message(Chain::Bitcoin, params, domain),
            WalletConnectionMethods::BtcSendTransfer => BitcoinRequestHandler::parse_send_transaction(Chain::Bitcoin, params),
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
        let request = WalletConnectRequest::mock("unknown_method", "{}", None);
        let result = WalletConnectRequestHandler::parse_request(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_operation_add_chain() {
        let request = WalletConnectRequest::mock("wallet_addEthereumChain", "{}", None);
        assert_eq!(
            WalletConnectRequestHandler::parse_request(request).unwrap(),
            WalletConnectAction::ChainOperation {
                operation: WalletConnectChainOperation::AddChain,
            }
        );
    }
}
