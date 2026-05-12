use std::{
    error::Error,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum SuiError {
    InvalidInput(String),
    InsufficientBalance,
    NoGasCoins,
}

impl Display for SuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(message) => write!(f, "{message}"),
            Self::InsufficientBalance => write!(f, "insufficient Sui coin balance"),
            Self::NoGasCoins => write!(f, "No SUI coins available for gas"),
        }
    }
}

impl Error for SuiError {}

impl SuiError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }
}
