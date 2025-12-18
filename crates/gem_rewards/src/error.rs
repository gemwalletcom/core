use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum RewardsError {
    Username(String),
    Referral(String),
    Redemption(String),
}

impl fmt::Display for RewardsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RewardsError::Username(msg) => write!(f, "{}", msg),
            RewardsError::Referral(msg) => write!(f, "{}", msg),
            RewardsError::Redemption(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for RewardsError {}
