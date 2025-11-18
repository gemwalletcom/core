pub use gem_evm::siwe::SiweMessage;
use primitives::Chain;

#[uniffi::remote(Record)]
pub struct SiweMessage {
    pub domain: String,
    pub address: String,
    pub uri: String,
    pub chain_id: u64,
    pub nonce: String,
    pub version: String,
    pub issued_at: String,
}

#[uniffi::export]
pub fn siwe_try_parse(raw: String) -> Option<SiweMessage> {
    SiweMessage::try_parse(&raw)
}

#[uniffi::export]
pub fn siwe_validate(message: SiweMessage, chain: Chain) -> Result<(), crate::GemstoneError> {
    message.validate(chain).map_err(|e| crate::GemstoneError::AnyError { msg: e })
}
