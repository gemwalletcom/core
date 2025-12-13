use alloy_primitives::hex;
use gem_hash::keccak::keccak256;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use primitives::{AuthMessage, ChainType};

pub struct AuthMessageData {
    pub message: String,
    pub hash: [u8; 32],
}

pub fn create_auth_hash(auth_message: &AuthMessage) -> AuthMessageData {
    let message = serde_json::to_string(auth_message).unwrap_or_default();
    let hash = keccak256(message.as_bytes());
    AuthMessageData { message, hash }
}

pub fn verify_auth_signature(auth_message: &AuthMessage, signature: &str) -> bool {
    match auth_message.chain.chain_type() {
        ChainType::Ethereum => verify_ethereum_signature(auth_message, signature),
        _ => false, // TODO: Add support for other chain types
    }
}

fn verify_ethereum_signature(auth_message: &AuthMessage, signature: &str) -> bool {
    let data = create_auth_hash(auth_message);
    verify_hash_signature(&data.hash, signature, &auth_message.address)
}

fn verify_hash_signature(hash: &[u8; 32], signature: &str, expected_address: &str) -> bool {
    let Some(recovered) = recover_address_from_hash(hash, signature) else {
        return false;
    };
    recovered.eq_ignore_ascii_case(expected_address)
}

fn recover_address_from_hash(hash: &[u8; 32], signature: &str) -> Option<String> {
    let signature_bytes = hex::decode(signature.strip_prefix("0x").unwrap_or(signature)).ok()?;

    if signature_bytes.len() != 65 {
        return None;
    }

    let r_s = &signature_bytes[..64];
    let v = signature_bytes[64];

    let recovery_id = match v {
        27 | 0 => RecoveryId::from_byte(0),
        28 | 1 => RecoveryId::from_byte(1),
        _ => return None,
    }?;

    let signature = Signature::from_slice(r_s).ok()?;
    let recovered_key = VerifyingKey::recover_from_prehash(hash, &signature, recovery_id).ok()?;

    let public_key_bytes = recovered_key.to_encoded_point(false);
    let public_key_hash = keccak256(&public_key_bytes.as_bytes()[1..]);
    let address = format!("0x{}", hex::encode(&public_key_hash[12..]));

    Some(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;
    use primitives::{AuthNonce, Chain};

    const TEST_PRIVATE_KEY: [u8; 32] = [
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19,
        0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
    ];

    fn sign_auth_message(auth_message: &AuthMessage, signing_key: &SigningKey) -> String {
        let message = serde_json::to_string(auth_message).unwrap();
        let hash = keccak256(message.as_bytes());
        let (signature, recovery_id) = signing_key.sign_prehash_recoverable(&hash).unwrap();
        let mut sig_bytes = signature.to_bytes().to_vec();
        sig_bytes.push(recovery_id.to_byte() + 27);
        format!("0x{}", hex::encode(&sig_bytes))
    }

    fn get_address_from_key(signing_key: &SigningKey) -> String {
        let verifying_key = signing_key.verifying_key();
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let public_key_hash = keccak256(&public_key_bytes.as_bytes()[1..]);
        format!("0x{}", hex::encode(&public_key_hash[12..]))
    }

    #[test]
    fn test_verify_auth_signature_success() {
        let signing_key = SigningKey::from_bytes(&TEST_PRIVATE_KEY.into()).unwrap();
        let address = get_address_from_key(&signing_key);

        let auth_message = AuthMessage {
            chain: Chain::Ethereum,
            address: address.clone(),
            auth_nonce: AuthNonce {
                nonce: "test-nonce-123".to_string(),
                timestamp: 1734100000,
            },
        };

        let signature = sign_auth_message(&auth_message, &signing_key);
        assert!(verify_auth_signature(&auth_message, &signature));
    }

    #[test]
    fn test_verify_auth_signature_invalid() {
        let auth_message = AuthMessage {
            chain: Chain::Ethereum,
            address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            auth_nonce: AuthNonce {
                nonce: "test123".to_string(),
                timestamp: 1234567890,
            },
        };
        assert!(!verify_auth_signature(&auth_message, "0x"));
    }
}
