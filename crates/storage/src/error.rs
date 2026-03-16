use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum DatabaseError {
    NotFound { resource: &'static str, lookup: NotFoundLookup },
    Error(String),
}

#[derive(Debug, Clone)]
pub enum NotFoundLookup {
    Public(String),
    Internal(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::NotFound {
                resource,
                lookup: NotFoundLookup::Public(lookup),
            } => write!(f, "{resource} {lookup} not found"),
            DatabaseError::NotFound {
                resource,
                lookup: NotFoundLookup::Internal(_),
            } => write!(f, "{resource} not found"),
            DatabaseError::Error(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for DatabaseError {}

impl DatabaseError {
    pub fn not_found(resource: &'static str, lookup: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            lookup: NotFoundLookup::Public(lookup.into()),
        }
    }

    pub fn not_found_internal(resource: &'static str, lookup: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            lookup: NotFoundLookup::Internal(lookup.into()),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }
}

pub trait ResourceName {
    const RESOURCE_NAME: &'static str;
}

impl ResourceName for crate::models::AssetRow {
    const RESOURCE_NAME: &'static str = "Asset";
}

impl ResourceName for crate::models::ConfigRow {
    const RESOURCE_NAME: &'static str = "Config";
}

impl ResourceName for crate::models::DeviceRow {
    const RESOURCE_NAME: &'static str = "Device";
}

impl ResourceName for crate::models::FiatQuoteRow {
    const RESOURCE_NAME: &'static str = "FiatQuote";
}

impl ResourceName for crate::models::FiatRateRow {
    const RESOURCE_NAME: &'static str = "FiatRate";
}

impl ResourceName for crate::models::NftAssetRow {
    const RESOURCE_NAME: &'static str = "NFTAsset";
}

impl ResourceName for crate::models::NftCollectionRow {
    const RESOURCE_NAME: &'static str = "NFTCollection";
}

impl ResourceName for crate::models::ParserStateRow {
    const RESOURCE_NAME: &'static str = "ParserState";
}

impl ResourceName for crate::models::PriceRow {
    const RESOURCE_NAME: &'static str = "Price";
}

impl ResourceName for crate::models::RedemptionOptionFull {
    const RESOURCE_NAME: &'static str = "RewardRedemptionOption";
}

impl ResourceName for crate::models::RewardEventRow {
    const RESOURCE_NAME: &'static str = "RewardEvent";
}

impl ResourceName for crate::models::RewardRedemptionRow {
    const RESOURCE_NAME: &'static str = "RewardRedemption";
}

impl ResourceName for crate::models::RewardsRow {
    const RESOURCE_NAME: &'static str = "Rewards";
}

impl ResourceName for crate::models::ScanAddressRow {
    const RESOURCE_NAME: &'static str = "ScanAddress";
}

impl ResourceName for crate::models::TransactionRow {
    const RESOURCE_NAME: &'static str = "Transaction";
}

impl ResourceName for crate::models::UsernameRow {
    const RESOURCE_NAME: &'static str = "Username";
}

impl ResourceName for crate::models::WalletRow {
    const RESOURCE_NAME: &'static str = "Wallet";
}

impl ResourceName for crate::models::WalletAddressRow {
    const RESOURCE_NAME: &'static str = "WalletAddress";
}

impl ResourceName for crate::models::WalletSubscriptionRow {
    const RESOURCE_NAME: &'static str = "WalletSubscription";
}

impl From<diesel::result::Error> for DatabaseError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => DatabaseError::Error("Unexpected database record not found without lookup context".to_string()),
            e => DatabaseError::Error(e.to_string()),
        }
    }
}

pub trait DieselResultExt<T> {
    fn or_not_found(self, lookup: String) -> Result<T, DatabaseError>
    where
        T: ResourceName;
    fn or_not_found_internal(self, lookup: String) -> Result<T, DatabaseError>
    where
        T: ResourceName;
    fn or_not_found_for<R: ResourceName>(self, lookup: String) -> Result<T, DatabaseError>;
    fn or_not_found_internal_for<R: ResourceName>(self, lookup: String) -> Result<T, DatabaseError>;
}

impl<T> DieselResultExt<T> for Result<T, diesel::result::Error> {
    fn or_not_found(self, lookup: String) -> Result<T, DatabaseError>
    where
        T: ResourceName,
    {
        self.or_not_found_for::<T>(lookup)
    }

    fn or_not_found_internal(self, lookup: String) -> Result<T, DatabaseError>
    where
        T: ResourceName,
    {
        self.or_not_found_internal_for::<T>(lookup)
    }

    fn or_not_found_for<R: ResourceName>(self, lookup: String) -> Result<T, DatabaseError> {
        match self {
            Ok(value) => Ok(value),
            Err(diesel::result::Error::NotFound) => Err(DatabaseError::not_found(R::RESOURCE_NAME, lookup)),
            Err(error) => Err(error.into()),
        }
    }

    fn or_not_found_internal_for<R: ResourceName>(self, lookup: String) -> Result<T, DatabaseError> {
        match self {
            Ok(value) => Ok(value),
            Err(diesel::result::Error::NotFound) => Err(DatabaseError::not_found_internal(R::RESOURCE_NAME, lookup)),
            Err(error) => Err(error.into()),
        }
    }
}

impl From<std::num::ParseIntError> for DatabaseError {
    fn from(error: std::num::ParseIntError) -> Self {
        DatabaseError::Error(error.to_string())
    }
}

impl From<std::num::ParseFloatError> for DatabaseError {
    fn from(error: std::num::ParseFloatError) -> Self {
        DatabaseError::Error(error.to_string())
    }
}

impl From<std::str::ParseBoolError> for DatabaseError {
    fn from(error: std::str::ParseBoolError) -> Self {
        DatabaseError::Error(error.to_string())
    }
}

impl From<serde_json::Error> for DatabaseError {
    fn from(error: serde_json::Error) -> Self {
        DatabaseError::Error(error.to_string())
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(error: r2d2::Error) -> Self {
        DatabaseError::Error(error.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum ReferralValidationError {
    CodeDoesNotExist,
    DeviceAlreadyUsed,
    CannotReferSelf,
    EligibilityExpired(i64),
    RewardsNotEnabled(String),
    Database(DatabaseError),
}

impl fmt::Display for ReferralValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferralValidationError::CodeDoesNotExist => write!(f, "Referral code does not exist"),
            ReferralValidationError::DeviceAlreadyUsed => write!(f, "This device has already been used to apply a referral code"),
            ReferralValidationError::CannotReferSelf => write!(f, "Cannot use your own referral code"),
            ReferralValidationError::EligibilityExpired(days) => write!(f, "eligibility_expired: {} days", days),
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

#[cfg(test)]
mod tests {
    use super::{DatabaseError, DieselResultExt, NotFoundLookup};

    #[test]
    fn test_database_error_display_not_found() {
        assert_eq!(DatabaseError::not_found("Asset", "0x1233").to_string(), "Asset 0x1233 not found");
    }

    #[test]
    fn test_database_error_display_wallet_address_not_found() {
        assert_eq!(DatabaseError::not_found("WalletAddress", "solana").to_string(), "WalletAddress solana not found");
    }

    #[test]
    fn test_database_error_display_hides_internal_lookup() {
        let error = DatabaseError::not_found_internal("Wallet", "42");
        assert_eq!(error.to_string(), "Wallet not found");
        match error {
            DatabaseError::NotFound {
                resource,
                lookup: NotFoundLookup::Internal(lookup),
            } => {
                assert_eq!(resource, "Wallet");
                assert_eq!(lookup, "42");
            }
            _ => panic!("expected internal not found"),
        }
    }

    #[test]
    fn test_diesel_result_ext_not_found() {
        let result: Result<crate::models::WalletRow, _> = Err(diesel::result::Error::NotFound);
        let error = result.or_not_found("abc".to_string()).unwrap_err();

        assert!(error.is_not_found());
        assert_eq!(error.to_string(), "Wallet abc not found");
    }

    #[test]
    fn test_diesel_result_ext_not_found_internal() {
        let result: Result<crate::models::WalletRow, _> = Err(diesel::result::Error::NotFound);
        let error = result.or_not_found_internal("42".to_string()).unwrap_err();

        assert!(error.is_not_found());
        assert_eq!(error.to_string(), "Wallet not found");
        match error {
            DatabaseError::NotFound {
                resource,
                lookup: NotFoundLookup::Internal(lookup),
            } => {
                assert_eq!(resource, "Wallet");
                assert_eq!(lookup, "42");
            }
            _ => panic!("expected internal not found"),
        }
    }
}
