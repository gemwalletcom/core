use primitives::{Deeplink, UrlAction, WalletConnectLink};

#[uniffi::remote(Enum)]
pub enum UrlAction {
    Deeplink(Deeplink),
    WalletConnect(WalletConnectLink),
}

#[uniffi::export]
pub fn url_action(url: &str) -> Option<UrlAction> {
    UrlAction::from_url(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_action() {
        assert!(matches!(url_action("https://gemwallet.com/tokens/bitcoin"), Some(UrlAction::Deeplink(_))));
        assert!(matches!(url_action("gem://wc?sessionTopic=abc"), Some(UrlAction::WalletConnect(_))));
        assert_eq!(url_action("https://example.com"), None);
    }
}
