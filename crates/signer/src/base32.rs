use data_encoding::BASE32_NOPAD;
use primitives::SignerError;

#[derive(Clone, Copy)]
pub struct Base32Address([u8; 32]);

impl Base32Address {
    pub fn from_slice(bytes: &[u8]) -> Result<Self, SignerError> {
        if bytes.len() != 32 {
            return Err(SignerError::invalid_input("invalid base32 address payload"));
        }

        let mut payload = [0u8; 32];
        payload.copy_from_slice(bytes);
        Ok(Self(payload))
    }

    pub fn payload(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn hint(&self) -> &[u8] {
        &self.0[28..32]
    }
}

pub fn decode_base32(value: &[u8]) -> Option<Vec<u8>> {
    BASE32_NOPAD.decode(value).ok()
}
