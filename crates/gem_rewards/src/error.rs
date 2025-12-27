use std::error::Error;
use std::fmt;

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
