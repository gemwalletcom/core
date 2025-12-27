use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq, Hash)]
#[strum(serialize_all = "camelCase")]
pub enum ConfigKey {
    // Referral
    ReferralPerIpDaily,
    ReferralPerIpWeekly,
    ReferralPerUserDaily,
    ReferralPerUserWeekly,
    ReferralUseDailyLimit,
    ReferralIneligibleCountries,

    // Username
    UsernameCreationPerIp,

    // Redemption
    RedemptionPerUserDaily,
    RedemptionPerUserWeekly,

    // Fiat
    FiatValidateSubscription,

    // Transactions
    TransactionsMinAmountUsd,

    // Alerter
    AlerterPriceIncreasePercent,
    AlerterPriceDecreasePercent,
    AlerterInterval,

    // Pricer
    PricerTimer,
    PricerOutdated,

    // Search
    SearchAssetsUpdateInterval,
    SearchPerpetualsUpdateInterval,
    SearchNftsUpdateInterval,
}

impl ConfigKey {
    pub fn all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn default_value(&self) -> &'static str {
        match self {
            Self::ReferralPerIpDaily => "3",
            Self::ReferralPerIpWeekly => "10",
            Self::ReferralPerUserDaily => "5",
            Self::ReferralPerUserWeekly => "15",
            Self::ReferralUseDailyLimit => "1000",
            Self::ReferralIneligibleCountries => "[]",
            Self::UsernameCreationPerIp => "10",
            Self::RedemptionPerUserDaily => "1",
            Self::RedemptionPerUserWeekly => "3",
            Self::FiatValidateSubscription => "false",
            Self::TransactionsMinAmountUsd => "0.05",
            Self::AlerterPriceIncreasePercent => "8.0",
            Self::AlerterPriceDecreasePercent => "10.0",
            Self::AlerterInterval => "60s",
            Self::PricerTimer => "60s",
            Self::PricerOutdated => "7d",
            Self::SearchAssetsUpdateInterval => "30m",
            Self::SearchPerpetualsUpdateInterval => "30m",
            Self::SearchNftsUpdateInterval => "30m",
        }
    }
}
