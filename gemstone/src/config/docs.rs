#[derive(uniffi::Enum, Clone)]
pub enum DocsUrl {
    WhatIsWatchWallet,
    WhatIsSecretPhrase,
    WhatIsPrivateKey,
    HowToSecureSecretPhrase,

    TransactionStatus,
}
const DOCS_URL: &str = "https://docs.gemwallet.com";

pub fn get_docs_url(item: DocsUrl) -> String {
    let path = match item {
        DocsUrl::WhatIsWatchWallet => "/faq/watch-wallet/",
        DocsUrl::WhatIsSecretPhrase => "/faq/secret-recovery-phrase/",
        DocsUrl::WhatIsPrivateKey => "/faq/private-key/",
        DocsUrl::HowToSecureSecretPhrase => "/faq/secure-recovery-phrase/",
        DocsUrl::TransactionStatus => "/faq/transaction-status/",
    };
    format!("{}{}", DOCS_URL, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_docs_url() {
        assert_eq!(
            get_docs_url(DocsUrl::WhatIsSecretPhrase),
            "https://docs.gemwallet.com/faq/secret-recovery-phrase/"
        );
    }
}
