use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DatabaseError {
    NotFound,
    Error(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::NotFound => write!(f, "Resource not found"),
            DatabaseError::Error(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for DatabaseError {}

impl From<diesel::result::Error> for DatabaseError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => DatabaseError::NotFound,
            e => DatabaseError::Error(e.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReferralValidationError {
    CodeDoesNotExist,
    AlreadyUsed,
    DeviceAlreadyUsed,
    CannotReferSelf,
    RewardsNotEnabled(String),
    Database(DatabaseError),
}

impl fmt::Display for ReferralValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferralValidationError::CodeDoesNotExist => write!(f, "Referral code does not exist"),
            ReferralValidationError::AlreadyUsed => write!(f, "You already have a username and cannot apply a referral code"),
            ReferralValidationError::DeviceAlreadyUsed => write!(f, "This device has already been used to apply a referral code"),
            ReferralValidationError::CannotReferSelf => write!(f, "Cannot use your own referral code"),
            ReferralValidationError::RewardsNotEnabled(user) => write!(f, "Rewards are not enabled for {}", user),
            ReferralValidationError::Database(e) => write!(f, "{}", e),
        }
    }
}

impl Error for ReferralValidationError {}

impl From<DatabaseError> for ReferralValidationError {
    fn from(error: DatabaseError) -> Self {
        ReferralValidationError::Database(error)
    }
}

impl From<diesel::result::Error> for ReferralValidationError {
    fn from(error: diesel::result::Error) -> Self {
        ReferralValidationError::Database(error.into())
    }
}
