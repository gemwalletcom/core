#[derive(Debug, uniffi::Enum)]
pub enum SignDigestType {
    Sign,
    Eip191,
    Eip712,
    Base58,
}
