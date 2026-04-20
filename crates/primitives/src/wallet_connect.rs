use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WCEthereumTransaction {
    pub chain_id: Option<String>,
    pub from: String,
    pub to: String,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WCTonMessage {
    pub address: String,
    pub amount: String,
    pub payload: Option<String>,
    pub state_init: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct WCTonSendTransaction {
    pub valid_until: Option<u32>,
    pub messages: Vec<WCTonMessage>,
    pub r#from: Option<String>,
    pub network: Option<String>,
}

impl WCTonSendTransaction {
    pub fn from_bytes(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data).or_else(|_| {
            Ok(Self {
                valid_until: None,
                messages: serde_json::from_slice(data)?,
                r#from: None,
                network: None,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ton_send_transaction_from_bytes() {
        let request = WCTonSendTransaction::from_bytes(include_bytes!("../testdata/wc_ton_send_transaction.json")).unwrap();
        assert_eq!(request.valid_until, Some(123));
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.r#from.as_deref(), Some("EQD..."));
        assert_eq!(request.network.as_deref(), Some("-239"));

        let legacy = WCTonSendTransaction::from_bytes(include_bytes!("../testdata/wc_ton_send_transaction_legacy.json")).unwrap();
        assert_eq!(legacy.valid_until, None);
        assert_eq!(legacy.messages.len(), 1);
        assert_eq!(legacy.messages[0].amount, "2");
        assert_eq!(legacy.r#from, None);
        assert_eq!(legacy.network, None);
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WCBitcoinTransfer {
    pub account: String,
    pub recipient_address: String,
    pub amount: String,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: Option<String>,
    pub domain: String,
}
