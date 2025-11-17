use primitives::Chain;

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
    Base58,
    Siwe,
}

#[derive(Debug, uniffi::Record)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}
