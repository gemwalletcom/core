use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumString, EnumIter, PartialEq, Eq, Hash)]
#[strum(serialize_all = "camelCase")]
pub enum ConfigKey {
    // Referral
    ReferralPerDeviceDaily,
    ReferralPerIpDaily,
    ReferralPerIpWeekly,
    ReferralPerUserDaily,
    ReferralPerUserWeekly,
    ReferralPerVerifiedUserDaily,
    ReferralPerVerifiedUserWeekly,
    ReferralUseDailyLimit,
    ReferralIneligibleCountries,
    ReferralVerificationDelay,
    ReferralEligibility,

    // Username
    UsernameCreationPerIp,
    UsernameCreationPerDevice,
    UsernameCreationGlobalDailyLimit,
    UsernameCreationPerCountryDailyLimit,

    // Redemption
    RedemptionPerUserDaily,
    RedemptionPerUserWeekly,

    // Referral IP
    ReferralIpConfidenceScoreThreshold,
    ReferralBlockedIpTypes,
    ReferralBlockedIpTypePenalty,
    ReferralMaxAbuseScore,
    ReferralPenaltyIsps,
    ReferralPenaltyIspsScore,
    ReferralIpTorAllowed,

    // Referral Risk Scoring (global cross-referrer penalties)
    ReferralRiskScoreFingerprintMatchPerReferrer,
    ReferralRiskScoreFingerprintMatchMaxPenalty,
    ReferralRiskScoreIpReuse,
    ReferralRiskScoreIspModelMatch,
    ReferralRiskScoreDeviceIdReusePerReferrer,
    ReferralRiskScoreDeviceIdReuseMaxPenalty,
    ReferralRiskScoreIneligibleIpType,
    ReferralRiskScoreVerifiedUserReduction,
    ReferralRiskScoreMaxAllowed,
    ReferralRiskScoreLookback,
    ReferralRiskScoreSameReferrerPatternThreshold,
    ReferralRiskScoreSameReferrerPatternPenalty,
    ReferralRiskScoreSameReferrerFingerprintThreshold,
    ReferralRiskScoreSameReferrerFingerprintPenalty,
    ReferralRiskScoreSameReferrerDeviceModelThreshold,
    ReferralRiskScoreSameReferrerDeviceModelPenalty,
    ReferralRiskScoreDeviceModelRingThreshold,
    ReferralRiskScoreDeviceModelRingPenaltyPerMember,
    ReferralRiskScoreHighRiskPlatformStores,
    ReferralRiskScoreHighRiskPlatformStorePenalty,
    ReferralRiskScoreHighRiskCountries,
    ReferralRiskScoreHighRiskCountryPenalty,
    ReferralRiskScoreHighRiskLocales,
    ReferralRiskScoreHighRiskLocalePenalty,

    // Referral Abuse Detection
    ReferralAbuseDisableThreshold,
    ReferralAbuseAttemptPenalty,
    ReferralAbuseVerifiedThresholdMultiplier,
    ReferralAbuseLookback,
    ReferralAbuseMinReferralsToEvaluate,

    ReferralAbuseCountryRotationThreshold,
    ReferralAbuseCountryRotationPenalty,
    ReferralAbuseRingReferrersPerDeviceThreshold,
    ReferralAbuseRingReferrersPerFingerprintThreshold,
    ReferralAbuseRingPenalty,
    ReferralAbuseDeviceFarmingThreshold,
    ReferralAbuseDeviceFarmingPenalty,

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
            Self::ReferralPerDeviceDaily => "2",
            Self::ReferralPerIpDaily => "3",
            Self::ReferralPerIpWeekly => "10",
            Self::ReferralPerUserDaily => "5",
            Self::ReferralPerUserWeekly => "15",
            Self::ReferralPerVerifiedUserDaily => "10",
            Self::ReferralPerVerifiedUserWeekly => "30",
            Self::ReferralUseDailyLimit => "1000",
            Self::ReferralIneligibleCountries => "[]",
            Self::ReferralVerificationDelay => "24h",
            Self::ReferralEligibility => "7d",
            Self::UsernameCreationPerIp => "10",
            Self::UsernameCreationPerDevice => "1",
            Self::UsernameCreationGlobalDailyLimit => "1000",
            Self::UsernameCreationPerCountryDailyLimit => "100",
            Self::RedemptionPerUserDaily => "1",
            Self::RedemptionPerUserWeekly => "3",
            Self::ReferralIpConfidenceScoreThreshold => "10",
            Self::ReferralBlockedIpTypes => r#"["Data Center", "Web Hosting", "Transit"]"#,
            Self::ReferralBlockedIpTypePenalty => "100",
            Self::ReferralMaxAbuseScore => "60",
            Self::ReferralPenaltyIsps => r#"[]"#,
            Self::ReferralPenaltyIspsScore => "30",
            Self::ReferralIpTorAllowed => "false",
            Self::ReferralRiskScoreFingerprintMatchPerReferrer => "50",
            Self::ReferralRiskScoreFingerprintMatchMaxPenalty => "200",
            Self::ReferralRiskScoreIpReuse => "50",
            Self::ReferralRiskScoreIspModelMatch => "30",
            Self::ReferralRiskScoreDeviceIdReusePerReferrer => "50",
            Self::ReferralRiskScoreDeviceIdReuseMaxPenalty => "200",
            Self::ReferralRiskScoreIneligibleIpType => "100",
            Self::ReferralRiskScoreVerifiedUserReduction => "20",
            Self::ReferralRiskScoreMaxAllowed => "60",
            Self::ReferralRiskScoreLookback => "90d",
            Self::ReferralRiskScoreSameReferrerPatternThreshold => "3",
            Self::ReferralRiskScoreSameReferrerPatternPenalty => "40",
            Self::ReferralRiskScoreSameReferrerFingerprintThreshold => "2",
            Self::ReferralRiskScoreSameReferrerFingerprintPenalty => "60",
            Self::ReferralRiskScoreSameReferrerDeviceModelThreshold => "3",
            Self::ReferralRiskScoreSameReferrerDeviceModelPenalty => "50",
            Self::ReferralRiskScoreDeviceModelRingThreshold => "2",
            Self::ReferralRiskScoreDeviceModelRingPenaltyPerMember => "40",
            Self::ReferralRiskScoreHighRiskPlatformStores => "[]",
            Self::ReferralRiskScoreHighRiskPlatformStorePenalty => "20",
            Self::ReferralRiskScoreHighRiskCountries => "[]",
            Self::ReferralRiskScoreHighRiskCountryPenalty => "15",
            Self::ReferralRiskScoreHighRiskLocales => "[]",
            Self::ReferralRiskScoreHighRiskLocalePenalty => "10",
            Self::ReferralAbuseDisableThreshold => "200",
            Self::ReferralAbuseAttemptPenalty => "15",
            Self::ReferralAbuseVerifiedThresholdMultiplier => "2",
            Self::ReferralAbuseLookback => "7d",
            Self::ReferralAbuseMinReferralsToEvaluate => "2",
            Self::ReferralAbuseCountryRotationThreshold => "2",
            Self::ReferralAbuseCountryRotationPenalty => "50",
            Self::ReferralAbuseRingReferrersPerDeviceThreshold => "2",
            Self::ReferralAbuseRingReferrersPerFingerprintThreshold => "2",
            Self::ReferralAbuseRingPenalty => "80",
            Self::ReferralAbuseDeviceFarmingThreshold => "5",
            Self::ReferralAbuseDeviceFarmingPenalty => "10",
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
