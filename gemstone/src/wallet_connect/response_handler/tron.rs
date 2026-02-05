use crate::wallet_connect::handler_traits::ChainResponseHandler;
use crate::wallet_connect::response_handler::WalletConnectResponseType;

pub struct TronResponseHandler;

impl ChainResponseHandler for TronResponseHandler {
    fn encode_sign_message(signature: String) -> WalletConnectResponseType {
        let result = serde_json::json!({
            "signature": signature
        });
        WalletConnectResponseType::Object {
            json: serde_json::to_string(&result).unwrap_or_default(),
        }
    }

    fn encode_sign_transaction(transaction_id: String) -> WalletConnectResponseType {
        let mut value =
            serde_json::from_str::<serde_json::Value>(&transaction_id).unwrap_or_else(|_| serde_json::Value::String(transaction_id));
        if let serde_json::Value::Object(map) = &mut value {
            map.entry("result".to_string()).or_insert(serde_json::Value::Bool(true));
        }
        let json = match value {
            serde_json::Value::String(value) => value,
            _ => value.to_string(),
        };
        WalletConnectResponseType::Object { json }
    }

    fn encode_send_transaction(transaction_id: String) -> WalletConnectResponseType {
        let json = serde_json::json!({ "result": true, "txid": transaction_id }).to_string();
        WalletConnectResponseType::Object { json }
    }
}
