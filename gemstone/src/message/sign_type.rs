use crate::siwe::SiweMessage;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
    Base58,
    SuiPersonalMessage,
    Siwe,
}

#[derive(Debug, uniffi::Record)]
pub struct SignMessage {
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
    pub siwe: Option<SiweMessage>,
}
