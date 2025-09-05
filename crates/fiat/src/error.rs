use thiserror::Error;

#[derive(Error, Debug)]
pub enum FiatError {
    #[error("Purchase not allowed")]
    FiatPurchaseNotAllowed,

    #[error("Sell not allowed")]
    FiatSellNotAllowed,

    #[error("Minimum Amount is {0}")]
    MinimumAmount(f64),

    #[error("Amount {0} is below minimum {1}")]
    InsufficientAmount(f64, f64),

    #[error("Amount {0} exceeds maximum {1}")]
    ExcessiveAmount(f64, f64),

    #[error("Unsupported country: {0}")]
    UnsupportedCountry(String),

    #[error("Unsupported country {0} for an asset: {0}")]
    UnsupportedCountryAsset(String, String),

    #[error("Unsupported state: {0}")]
    UnsupportedState(String),
}
