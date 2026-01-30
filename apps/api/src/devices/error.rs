use std::fmt;

pub enum DeviceError {
    MissingHeader(&'static str),
    MissingParameter(&'static str),
    DeviceNotFound,
    WalletNotFound,
    DatabaseUnavailable,
    DatabaseError,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader(name) => write!(f, "Missing header: {}", name),
            Self::MissingParameter(name) => write!(f, "Missing parameter: {}", name),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::WalletNotFound => write!(f, "Wallet not found"),
            Self::DatabaseUnavailable => write!(f, "Database not available"),
            Self::DatabaseError => write!(f, "Database error"),
        }
    }
}
