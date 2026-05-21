use crate::{Deeplink, WalletConnectLink};

#[derive(Debug, Clone, PartialEq)]
pub enum UrlAction {
    Deeplink { deeplink: Deeplink },
    WalletConnect { link: WalletConnectLink },
}

impl UrlAction {
    pub fn from_url(url: &str) -> Option<Self> {
        if let Some(link) = WalletConnectLink::from_url(url) {
            return Some(Self::WalletConnect { link });
        }
        Deeplink::from_url(url).map(|deeplink| Self::Deeplink { deeplink })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, Chain};

    #[test]
    fn test_from_url() {
        assert_eq!(
            UrlAction::from_url("https://gemwallet.com/tokens/bitcoin"),
            Some(UrlAction::Deeplink {
                deeplink: Deeplink::Asset {
                    asset_id: AssetId::from_chain(Chain::Bitcoin),
                },
            })
        );
        assert_eq!(
            UrlAction::from_url("gem://wc?sessionTopic=abc123"),
            Some(UrlAction::WalletConnect {
                link: WalletConnectLink::Session { topic: "abc123".to_string() },
            })
        );
        assert_eq!(
            UrlAction::from_url("wc:topic@2?relay-protocol=irn&symKey=abc"),
            Some(UrlAction::WalletConnect {
                link: WalletConnectLink::Connect {
                    uri: "wc:topic@2?relay-protocol=irn&symKey=abc".to_string(),
                },
            })
        );
        assert_eq!(UrlAction::from_url("https://example.com/tokens/bitcoin"), None);
        assert_eq!(UrlAction::from_url("not a url"), None);
    }
}
