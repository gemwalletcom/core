use std::error::Error;
use std::fmt;

use primitives::ConfigKey;
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
