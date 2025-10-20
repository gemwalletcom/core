#[derive(Debug, uniffi::Enum)]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
    Base58,
    SuiPersonalMessage,
}

#[derive(Debug, uniffi::Record)]
pub struct SignMessage {
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}
