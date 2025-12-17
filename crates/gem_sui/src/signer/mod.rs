mod chain_signer;
mod signature;

pub use chain_signer::{SuiChainSigner, sign_from_metadata, sign_message_bytes};
pub use signature::{SUI_PERSONAL_MESSAGE_SIGNATURE_LEN, sign_digest, sign_personal_message};
