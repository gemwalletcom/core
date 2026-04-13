mod error;

pub use error::AddressError;

/// Common trait for blockchain addresses.
pub trait Address: Sized {
    fn try_parse(address: &str) -> Option<Self>;

    fn as_bytes(&self) -> &[u8];

    fn encode(&self) -> String;

    fn from_str(address: &str) -> Result<Self, AddressError> {
        Self::try_parse(address).ok_or_else(|| AddressError::new("invalid address"))
    }

    fn is_valid(address: &str) -> bool {
        Self::try_parse(address).is_some()
    }
}
