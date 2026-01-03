use std::error::Error;
use std::fmt;

use localizer::LanguageLocalizer;
use primitives::{ConfigKey, Localize};
use storage::{DatabaseError, ReferralValidationError};

#[derive(Debug)]
pub enum RewardsError {
    Username(String),
    Referral(String),
}

impl fmt::Display for RewardsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewardsError::Username(msg) => write!(f, "{}", msg),
            RewardsError::Referral(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for RewardsError {}

#[derive(Debug)]
pub enum ReferralError {
    Validation(ReferralValidationError),
    ReferrerLimitReached(String),
    RiskScoreExceeded { score: i64, max_allowed: i64 },
    IpTorNotAllowed,
    IpCountryIneligible(String),
    LimitReached(ConfigKey),
    Database(DatabaseError),
}

impl fmt::Display for ReferralError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferralError::Validation(e) => write!(f, "{}", e),
            ReferralError::ReferrerLimitReached(period) => write!(f, "Referrer {} limit reached", period),
            ReferralError::RiskScoreExceeded { score, max_allowed } => write!(f, "risk_score: {} (max allowed: {})", score, max_allowed),
            ReferralError::IpTorNotAllowed => write!(f, "ip_tor_not_allowed"),
            ReferralError::IpCountryIneligible(country) => write!(f, "ip_country_ineligible: {}", country),
            ReferralError::LimitReached(key) => write!(f, "limit_reached: {}", key.as_ref()),
            ReferralError::Database(e) => write!(f, "{}", e),
        }
    }
}

impl Error for ReferralError {}

impl ReferralError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation(_) | Self::ReferrerLimitReached(_) => self.to_string(),
            Self::RiskScoreExceeded { .. } | Self::IpTorNotAllowed | Self::IpCountryIneligible(_) | Self::LimitReached(_) | Self::Database(_) => {
                "Unable to verify referral eligibility".to_string()
            }
        }
    }
}

impl Localize for ReferralError {
    fn localize(&self, locale: &str) -> String {
        let localizer = LanguageLocalizer::new_with_language(locale);
        match self {
            Self::Validation(ReferralValidationError::CodeDoesNotExist) => localizer.rewards_error_referral_code_not_exist(),
            Self::Validation(ReferralValidationError::AlreadyUsed) => localizer.rewards_error_referral_already_used(),
            Self::Validation(ReferralValidationError::DeviceAlreadyUsed) => localizer.rewards_error_referral_device_already_used(),
            Self::Validation(ReferralValidationError::CannotReferSelf) => localizer.rewards_error_referral_cannot_refer_self(),
            Self::Validation(ReferralValidationError::RewardsNotEnabled(_)) => localizer.rewards_error_referral_rewards_not_enabled(),
            Self::Validation(ReferralValidationError::Database(_)) => localizer.errors_generic(),
            Self::ReferrerLimitReached(_) => localizer.rewards_error_referral_referrer_limit_reached(),
            Self::IpCountryIneligible(country) => localizer.rewards_error_referral_country_ineligible(country),
            Self::RiskScoreExceeded { .. } | Self::IpTorNotAllowed | Self::LimitReached(_) => localizer.rewards_error_referral_limit_reached(),
            Self::Database(_) => localizer.errors_generic(),
        }
    }
}

impl From<ReferralValidationError> for ReferralError {
    fn from(error: ReferralValidationError) -> Self {
        ReferralError::Validation(error)
    }
}

impl From<DatabaseError> for ReferralError {
    fn from(error: DatabaseError) -> Self {
        ReferralError::Database(error)
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ReferralError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        ReferralError::Database(DatabaseError::Error(error.to_string()))
    }
}

#[derive(Debug)]
pub enum RewardsRedemptionError {
    NotEligible(String),
    DailyLimitReached,
    WeeklyLimitReached,
    NotEnoughPoints,
    OptionNotAvailable,
    NoUsername,
}

impl fmt::Display for RewardsRedemptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewardsRedemptionError::NotEligible(msg) => write!(f, "{}", msg),
            RewardsRedemptionError::DailyLimitReached => write!(f, "Daily redemption limit reached"),
            RewardsRedemptionError::WeeklyLimitReached => write!(f, "Weekly redemption limit reached"),
            RewardsRedemptionError::NotEnoughPoints => write!(f, "Not enough points"),
            RewardsRedemptionError::OptionNotAvailable => write!(f, "Redemption option is no longer available"),
            RewardsRedemptionError::NoUsername => write!(f, "No username found for address"),
        }
    }
}

impl Error for RewardsRedemptionError {}
