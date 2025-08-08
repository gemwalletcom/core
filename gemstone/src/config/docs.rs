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
    TronMultiSignature,
    RootedDevice,
    PriceImpact,
    TokenApproval,
    Slippage,
    SwapProvider,
    FiatProvider,
    StakingAPR,
    StakingStatus,
    StakingValidator,
    AccountMinimalBalance,
    TokenVerification,
    AddCustomToken,
    WalletConnect,
    HowStoreSecretPhrase,
    NoQuotes,
    Staking,
    PerpetualsFundingRate,
    PerpetualsLiquidationPrice,
    PerpetualsOpenInterest,
    PerpetualsFundingPayments,
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
        DocsUrl::TronMultiSignature => "/guides/trx-multisig-scam/",
        DocsUrl::RootedDevice => "/guides/secure-wallet/rooted-device/",
        DocsUrl::PriceImpact => "/faq/price-impact/",
        DocsUrl::TokenApproval => "/faq/token-approval/",
        DocsUrl::Slippage => "/faq/slippage/",
        DocsUrl::SwapProvider => "/faq/swap-provider/",
        DocsUrl::FiatProvider => "/faq/fiat-provider/",
        DocsUrl::StakingAPR => "/faq/staking-apr/",
        DocsUrl::StakingStatus => "/faq/staking-status/",
        DocsUrl::StakingValidator => "/faq/staking-validator/",
        DocsUrl::AccountMinimalBalance => "/faq/account-minimal-balance/",
        DocsUrl::TokenVerification => "/faq/token-verification/",
        DocsUrl::AddCustomToken => "/guides/add-token/",
        DocsUrl::WalletConnect => "/guides/walletconnect/",
        DocsUrl::HowStoreSecretPhrase => "/faq/secure-recovery-phrase/#how-to-secure-my-secret-phrase/",
        DocsUrl::NoQuotes => "/troubleshoot/quote-error/",
        DocsUrl::Staking => "/defi/stake/",
        DocsUrl::PerpetualsFundingRate => "/defi/perps/perps-terms/#what-is-perpetual-funding/",
        DocsUrl::PerpetualsLiquidationPrice => "/defi/perps/perps-terms/#what-is-a-perpetual-liquidation-price/",
        DocsUrl::PerpetualsOpenInterest => "/defi/perps/perps-terms/#what-is-a-perpetual-open-interest/",
        DocsUrl::PerpetualsFundingPayments => "/defi/perps/perps-terms/#what-is-perpetual-funding-payments/",
    };
    format!("{DOCS_URL}{path}")
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
