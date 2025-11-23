use primitives::Chain;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum SignDigestType {
    Eip191,
    Eip712,
    Base58,
    SuiPersonal,
    Siwe,
}

#[derive(Debug, uniffi::Record)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}
