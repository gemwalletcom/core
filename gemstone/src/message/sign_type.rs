use crate::siwe::SiweMessage;
use primitives::Chain;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
    Base58,
    Siwe { message: SiweMessage },
}

impl SignDigestType {
    pub fn siwe_message(&self) -> Option<&SiweMessage> {
        match self {
            Self::Siwe { message } => Some(message),
            _ => None,
        }
    }
}

#[derive(Debug, uniffi::Record)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}
