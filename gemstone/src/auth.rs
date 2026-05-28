use crate::GemstoneError;
use gem_auth::create_auth_hash;
use primitives::{AuthMessage, AuthNonce, Chain, hex::encode_with_0x};
use signer::Signer;
use zeroize::Zeroizing;

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

#[uniffi::export]
pub fn sign_auth_message_hash(hash: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
    if hash.len() != 32 {
        return Err(GemstoneError::from("Invalid auth message hash"));
    }
    let private_key = Zeroizing::new(private_key);
    let signature = Signer::sign_ethereum_digest(&hash, &private_key)?;
    Ok(encode_with_0x(&signature))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, keccak256};
    use gem_auth::verify_auth_signature;
    use signer::secp256k1_uncompressed_public_key;

    const TEST_PRIVATE_KEY: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
        0x1d, 0x1e, 0x1f, 0x20,
    ];

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
        let message = create_auth_message(Chain::Ethereum, &address, auth_nonce);

        let signature = sign_auth_message_hash(message.hash, TEST_PRIVATE_KEY.to_vec()).unwrap();

        assert!(verify_auth_signature(&auth_message, &signature));
    }

    #[test]
    fn test_sign_auth_message_hash_rejects_invalid_hash_length() {
        let result = sign_auth_message_hash(vec![0; 31], TEST_PRIVATE_KEY.to_vec());

        assert!(result.is_err());
    }

    fn address_from_private_key(private_key: &[u8]) -> String {
        let public_key = secp256k1_uncompressed_public_key(private_key).unwrap();
        let hash = keccak256(&public_key[1..]);
        Address::from_slice(&hash[12..]).to_checksum(None)
    }
}
