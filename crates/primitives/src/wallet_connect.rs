use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use url::Url;

const WALLET_CONNECT_SCHEME: &str = "wc";
const WALLET_CONNECT_HOST: &str = "wc";
const GEM_SCHEME: &str = "gem";

const QUERY_URI: &str = "uri";
const QUERY_SESSION_TOPIC: &str = "sessionTopic";
const QUERY_REQUEST_ID: &str = "requestId";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: Option<String>,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WalletConnectLink {
    Connect { uri: String },
    Request,
    Session { topic: String },
}

impl WalletConnectLink {
    pub fn from_url(url: &str) -> Option<Self> {
        let parsed = Url::parse(url).ok()?;
        match parsed.scheme() {
            WALLET_CONNECT_SCHEME => Some(Self::session_or_request(&parsed).unwrap_or_else(|| WalletConnectLink::Connect { uri: url.to_string() })),
            GEM_SCHEME if parsed.host_str() == Some(WALLET_CONNECT_HOST) => match query_value(&parsed, QUERY_URI).filter(|uri| !uri.is_empty()) {
                Some(uri) => Some(WalletConnectLink::Connect { uri }),
                None => Self::session_or_request(&parsed),
            },
            _ => None,
        }
    }

    fn session_or_request(url: &Url) -> Option<Self> {
        if let Some(topic) = query_value(url, QUERY_SESSION_TOPIC).filter(|topic| !topic.is_empty()) {
            Some(WalletConnectLink::Session { topic })
        } else if query_value(url, QUERY_REQUEST_ID).is_some() {
            Some(WalletConnectLink::Request)
        } else {
            None
        }
    }
}

fn query_value(url: &Url, key: &str) -> Option<String> {
    url.query_pairs().find(|(query_key, _)| query_key.as_ref() == key).map(|(_, value)| value.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_connect_link_from_url() {
        assert_eq!(
            WalletConnectLink::from_url("wc:abc@2?relay-protocol=irn&symKey=123"),
            Some(WalletConnectLink::Connect {
                uri: "wc:abc@2?relay-protocol=irn&symKey=123".to_string(),
            })
        );
        assert_eq!(WalletConnectLink::from_url("wc:abc@2?requestId"), Some(WalletConnectLink::Request));
        assert_eq!(WalletConnectLink::from_url("wc:abc@2?requestId=123"), Some(WalletConnectLink::Request));
        assert_eq!(
            WalletConnectLink::from_url("gem://wc?uri=wc:topic@2"),
            Some(WalletConnectLink::Connect { uri: "wc:topic@2".to_string() })
        );
        assert_eq!(
            WalletConnectLink::from_url("gem://wc?uri=wc%3Atopic%402%3Frelay-protocol%3Dirn%26symKey%3Dabc"),
            Some(WalletConnectLink::Connect {
                uri: "wc:topic@2?relay-protocol=irn&symKey=abc".to_string(),
            })
        );
        assert_eq!(WalletConnectLink::from_url("gem://wc?requestId=1"), Some(WalletConnectLink::Request));
        assert_eq!(
            WalletConnectLink::from_url("gem://wc?sessionTopic=abc123"),
            Some(WalletConnectLink::Session { topic: "abc123".to_string() })
        );
        assert_eq!(WalletConnectLink::from_url("gem://wc?sessionTopic="), None);
        assert_eq!(WalletConnectLink::from_url("gem://asset/solana"), None);
        assert_eq!(WalletConnectLink::from_url("https://gemwallet.com/tokens/bitcoin"), None);
    }
}
