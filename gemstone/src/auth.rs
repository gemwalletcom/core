use crate::GemstoneError;
use gem_auth::create_auth_hash;
use primitives::{AuthMessage, AuthNonce, Chain, hex::encode_with_0x};
use signer::Signer;
use zeroize::Zeroizing;

const AUTH_SIGNING_BYTES_LENGTH: usize = 32;

pub type GemAuthNonce = AuthNonce;

#[uniffi::remote(Record)]
pub struct GemAuthNonce {
    pub nonce: String,
    pub timestamp: u32,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemAuthMessage {
    pub message: String,
    pub hash: Vec<u8>,
}

#[uniffi::export]
pub fn create_auth_message(address: &str, auth_nonce: GemAuthNonce) -> GemAuthMessage {
    let auth_message = AuthMessage {
        chain: Chain::Ethereum,
        address: address.to_string(),
        auth_nonce,
    };
    let data = create_auth_hash(&auth_message);
    GemAuthMessage {
        message: data.message,
        hash: data.hash.to_vec(),
    }
}

#[uniffi::export]
pub fn sign_auth_message_hash(hash: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    let private_key = Zeroizing::new(private_key);
    if hash.len() != AUTH_SIGNING_BYTES_LENGTH || private_key.len() != AUTH_SIGNING_BYTES_LENGTH {
        return Err(GemstoneError::from("Invalid auth message signing input"));
    }
    let signature = Signer::sign_ethereum_digest(&hash, private_key.as_slice())?;
    Ok(encode_with_0x(&signature))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, keccak256};
    use gem_auth::verify_auth_signature;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use signer::secp256k1_uncompressed_public_key;

    #[test]
    fn test_sign_auth_message_hash() {
        let address = address_from_private_key(&TEST_PRIVATE_KEY);
        let auth_nonce = AuthNonce {
            nonce: "test-nonce-123".to_string(),
            timestamp: 1734100000,
        };
        let auth_message = AuthMessage {
            chain: Chain::Ethereum,
            address: address.clone(),
            auth_nonce: auth_nonce.clone(),
        };
        let message = create_auth_message(&address, auth_nonce);

        let signature = sign_auth_message_hash(message.hash, TEST_PRIVATE_KEY.to_vec()).unwrap();

        assert!(verify_auth_signature(&auth_message, &signature));
    }

    #[test]
    fn test_sign_auth_message_hash_rejects_invalid_input_length() {
        assert!(sign_auth_message_hash(vec![0; 31], TEST_PRIVATE_KEY.to_vec()).is_err());
        assert!(sign_auth_message_hash(vec![0; 32], vec![0; 31]).is_err());
    }

    fn address_from_private_key(private_key: &[u8]) -> String {
        let public_key = secp256k1_uncompressed_public_key(private_key).unwrap();
        let hash = keccak256(&public_key[1..]);
        Address::from_slice(&hash[12..]).to_checksum(None)
    }
}
