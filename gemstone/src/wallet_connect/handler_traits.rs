use crate::wallet_connect::actions::WalletConnectAction;
use crate::wallet_connect::response_handler::WalletConnectResponseType;
use primitives::Chain;
use serde_json::Value;

pub trait ChainRequestHandler {
    fn parse_sign_message(chain: Chain, params: Value) -> Result<WalletConnectAction, String>;
    fn parse_sign_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String>;
    fn parse_send_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String>;
}

pub trait ChainResponseHandler {
    fn encode_sign_message(signature: String) -> WalletConnectResponseType;
    fn encode_sign_transaction(transaction_id: String) -> WalletConnectResponseType;
    fn encode_send_transaction(transaction_id: String) -> WalletConnectResponseType;
}
