use std::fmt;

#[derive(Debug)]
pub struct AddressError {
    pub message: String,
}

impl AddressError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AddressError {}

impl From<AddressError> for String {
    fn from(err: AddressError) -> Self {
        err.message
    }
}
