use gem_auth::create_auth_hash;
use primitives::{AuthMessage, AuthNonce, Chain};

pub type GemAuthNonce = AuthNonce;

#[uniffi::remote(Record)]
pub struct GemAuthNonce {
    pub nonce: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAuthMessage {
    pub message: String,
    pub hash: Vec<u8>,
}

#[uniffi::export]
pub fn create_auth_message(chain: Chain, address: &str, auth_nonce: GemAuthNonce) -> GemAuthMessage {
    let auth_message = AuthMessage {
        chain,
        address: address.to_string(),
        auth_nonce,
    };
    let data = create_auth_hash(&auth_message);
    GemAuthMessage {
        message: data.message,
        hash: data.hash.to_vec(),
    }
}
