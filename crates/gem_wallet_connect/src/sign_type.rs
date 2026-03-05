use primitives::Chain;

#[derive(Debug, Clone, PartialEq)]
pub enum SignDigestType {
    Eip191,
    Eip712,
    Base58,
    SuiPersonal,
    Siwe,
    TonPersonal,
    BitcoinPersonal,
    TronPersonal,
}

#[derive(Debug)]
pub struct SignMessage {
    pub chain: Chain,
    pub sign_type: SignDigestType,
    pub data: Vec<u8>,
}
