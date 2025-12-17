use primitives::SignerError;
use signer::Signer;

use super::types::TonSignMessageData;

pub fn sign_personal(data: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SignerError> {
    let ton_data = TonSignMessageData::from_bytes(data)?;
    let payload = ton_data.get_payload()?;
    let digest = payload.hash();

    Signer::sign_ed25519_with_public_key(&digest, private_key).map_err(|e| SignerError::InvalidInput(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_ton_personal() {
        let payload = serde_json::json!({"type": "text", "text": "Hello TON"});
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string());
        let data = ton_data.to_bytes();

        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").expect("valid hex");

        let (signature, public_key) = sign_personal(&data, &private_key).expect("signing succeeds");

        assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");
        assert_eq!(public_key.len(), 32, "Ed25519 public key should be 32 bytes");
    }

    #[test]
    fn test_sign_ton_personal_rejects_invalid_key() {
        let payload = serde_json::json!({"type": "text", "text": "Hello TON"});
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string());
        let data = ton_data.to_bytes();

        let result = sign_personal(&data, &[0u8; 16]);
        assert!(result.is_err());
    }
}
