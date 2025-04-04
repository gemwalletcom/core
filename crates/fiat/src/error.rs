use thiserror::Error;

#[derive(Error, Debug)]
pub enum FiatError {
    #[error("Purchase not allowed")]
    FiatPurchaseNotAllowed,

    #[error("Sell not allowed")]
    FiatSellNotAllowed,

    #[error("Unsupported country: {0}")]
    UnsupportedCountry(String),

    #[error("Unsupported country {0} for an asset: {0}")]
    UnsupportedCountryAsset(String, String),

    #[error("Unsupported state: {0}")]
    UnsupportedState(String),
}
