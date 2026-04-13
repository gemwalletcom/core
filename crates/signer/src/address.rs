use primitives::SignerError;

#[derive(Clone, Copy)]
pub struct Base32Address([u8; 32]);

impl Base32Address {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, SignerError> {
        let payload: [u8; 32] = bytes.try_into().map_err(|_| SignerError::invalid_input("invalid base32 address payload"))?;
        Ok(Self(payload))
    }

    pub fn payload(&self) -> &[u8; 32] {
        &self.0
    }
}
