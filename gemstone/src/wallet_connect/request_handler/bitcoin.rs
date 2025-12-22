use crate::message::sign_type::SignDigestType;
use crate::wallet_connect::actions::{WalletConnectAction, WalletConnectTransactionType};
use crate::wallet_connect::handler_traits::ChainRequestHandler;
use gem_bitcoin::signer::BitcoinSignMessageData;
use primitives::wallet_connect::WCBitcoinTransfer;
use primitives::{Chain, TransferDataOutputType};
use serde_json::Value;

pub struct BitcoinRequestHandler;

impl ChainRequestHandler for BitcoinRequestHandler {
    fn parse_sign_message(chain: Chain, params: Value, _domain: &str) -> Result<WalletConnectAction, String> {
        let message = params
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or("Missing message parameter")?;
        let address = params
            .get("address")
            .and_then(|v| v.as_str())
            .ok_or("Missing address parameter")?;

        let btc_data = BitcoinSignMessageData::new(message.to_string(), address.to_string());
        let data = serde_json::to_string(&btc_data).map_err(|e| e.to_string())?;

        Ok(WalletConnectAction::SignMessage {
            chain,
            sign_type: SignDigestType::BitcoinPersonal,
            data,
        })
    }

    fn parse_sign_transaction(_chain: Chain, _params: Value) -> Result<WalletConnectAction, String> {
        Err("Bitcoin signTransaction not supported, use sendTransfer instead".to_string())
    }

    fn parse_send_transaction(chain: Chain, params: Value) -> Result<WalletConnectAction, String> {
        let transfer: WCBitcoinTransfer = serde_json::from_value(params).map_err(|e| e.to_string())?;
        let data = serde_json::to_string(&transfer).map_err(|e| e.to_string())?;

        Ok(WalletConnectAction::SendTransaction {
            chain,
            transaction_type: WalletConnectTransactionType::Bitcoin {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sign_message() {
        let params = serde_json::json!({
            "account": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
            "address": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
            "message": "Hello Bitcoin"
        });
        let action = BitcoinRequestHandler::parse_sign_message(Chain::Bitcoin, params, "").unwrap();
        let WalletConnectAction::SignMessage { chain, sign_type, data } = action else {
            panic!("Expected SignMessage action")
        };
        assert_eq!(chain, Chain::Bitcoin);
        assert_eq!(sign_type, SignDigestType::BitcoinPersonal);

        let parsed = BitcoinSignMessageData::from_bytes(data.as_bytes()).unwrap();
        assert_eq!(parsed.message, "Hello Bitcoin");
        assert_eq!(parsed.address, "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh");
    }

    #[test]
    fn test_parse_send_transaction() {
        let params = serde_json::json!({
            "account": "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
            "recipientAddress": "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq",
            "amount": "100000"
        });
        let action = BitcoinRequestHandler::parse_send_transaction(Chain::Bitcoin, params).unwrap();

        let WalletConnectAction::SendTransaction { chain, transaction_type, data } = action else {
            panic!("Expected SendTransaction action")
        };
        assert_eq!(chain, Chain::Bitcoin);
        let WalletConnectTransactionType::Bitcoin {
            output_type: TransferDataOutputType::EncodedTransaction,
        } = transaction_type
        else {
            panic!("Expected Bitcoin transaction type with EncodedTransaction output")
        };
        let parsed: WCBitcoinTransfer = serde_json::from_str(&data).unwrap();
        assert_eq!(parsed.recipient_address, "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq");
        assert_eq!(parsed.amount, "100000");
    }
}
