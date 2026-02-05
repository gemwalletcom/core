use std::fmt;

pub enum DeviceError {
    MissingHeader(&'static str),
    InvalidDeviceId,
    InvalidTimestamp,
    TimestampExpired,
    InvalidSignature,
    DeviceNotFound,
    WalletNotFound,
    DatabaseUnavailable,
    DatabaseError,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHeader(name) => write!(f, "Missing header: {}", name),
            Self::InvalidDeviceId => write!(f, "Invalid device ID"),
            Self::InvalidTimestamp => write!(f, "Invalid timestamp"),
            Self::TimestampExpired => write!(f, "Timestamp expired"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::DeviceNotFound => write!(f, "Device not found"),
            Self::WalletNotFound => write!(f, "Wallet not found"),
            Self::DatabaseUnavailable => write!(f, "Database not available"),
            Self::DatabaseError => write!(f, "Database error"),
        }
    }
}
