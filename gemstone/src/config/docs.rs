#[derive(uniffi::Enum, Clone)]
pub enum DocsUrl {
    Start,
    WhatIsWatchWallet,
    WhatIsSecretPhrase,
    WhatIsPrivateKey,
    HowToSecureSecretPhrase,
    TransactionStatus,
    NetworkFees,
    StakingLockTime,
    TronMultiSignature
}
const DOCS_URL: &str = "https://docs.gemwallet.com";

pub fn get_docs_url(item: DocsUrl) -> String {
    let path = match item {
        DocsUrl::Start => "/",
        DocsUrl::WhatIsWatchWallet => "/faq/watch-wallet/",
        DocsUrl::WhatIsSecretPhrase => "/faq/secret-recovery-phrase/",
        DocsUrl::WhatIsPrivateKey => "/faq/private-key/",
        DocsUrl::HowToSecureSecretPhrase => "/faq/secure-recovery-phrase/",
        DocsUrl::TransactionStatus => "/faq/transaction-status/",
        DocsUrl::NetworkFees => "/faq/network-fees/",
        DocsUrl::StakingLockTime => "/faq/lock-time/",
        DocsUrl::TronMultiSignature => "/guides/trx-multisig-scam/"
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
