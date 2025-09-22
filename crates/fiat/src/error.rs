use std::fmt;

#[derive(Debug)]
pub enum FiatError {
    FiatPurchaseNotAllowed,
    FiatSellNotAllowed,
    MinimumAmount(f64),
    InsufficientAmount(f64, f64),
    ExcessiveAmount(f64, f64),
    UnsupportedCountry(String),
    UnsupportedCountryAsset(String, String),
    UnsupportedState(String),
}

impl fmt::Display for FiatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FiatPurchaseNotAllowed => write!(f, "Purchase not allowed"),
            Self::FiatSellNotAllowed => write!(f, "Sell not allowed"),
            Self::MinimumAmount(amount) => write!(f, "Minimum Amount is {}", amount),
            Self::InsufficientAmount(amount, min) => write!(f, "Amount {} is below minimum {}", amount, min),
            Self::ExcessiveAmount(amount, max) => write!(f, "Amount {} exceeds maximum {}", amount, max),
            Self::UnsupportedCountry(country) => write!(f, "Unsupported country: {}", country),
            Self::UnsupportedCountryAsset(country, asset) => write!(f, "Unsupported country {} for an asset: {}", country, asset),
            Self::UnsupportedState(state) => write!(f, "Unsupported state: {}", state),
        }
    }
}

impl std::error::Error for FiatError {}
