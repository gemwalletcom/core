#[derive(uniffi::Enum, Clone)]
pub enum DocsItem {
    WhatIsWatchWallet,
    WhatIsSecretPhrase,
    WhatIsPrivateKey,
    HowToSecureSecretPhrase,

    TransactionStatus,
}

const DOCS_URL: &str = "https://docs.gemwallet.com";

pub fn get_docs_url(item: DocsItem) -> String {
    let path = match item {
        DocsItem::WhatIsWatchWallet => "/faqs/watch-wallet/",
        DocsItem::WhatIsSecretPhrase => "/faqs/secret-recovery-phrase/",
        DocsItem::WhatIsPrivateKey => "/faqs/private-key/",
        DocsItem::HowToSecureSecretPhrase => "/faqs/secure-recovery-phrase/",
        DocsItem::TransactionStatus => "/faqs/transaction-status/",
    };
    format!("{}{}", DOCS_URL, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_docs_url() {
        assert_eq!(
            get_docs_url(DocsItem::WhatIsSecretPhrase),
            "https://docs.gemwallet.com/faqs/secret-recovery-phrase/"
        );
    }
}
