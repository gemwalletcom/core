use thiserror::Error;

#[derive(Error, Debug)]
pub enum FiatError {
    #[error("Purchase not allowed")]
    FiatPurchaseNotAllowed,

    #[error("Sell not allowed")]
    FiatSellNotAllowed,

    #[error("Unsupported country: {0}")]
    UnsupportedCountry(String),

    #[error("Unsupported state: {0}")]
    UnsupportedState(String),
}
