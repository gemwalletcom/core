use ed25519_dalek::Signer as DalekSigner;
use primitives::SignerError;
use zeroize::Zeroizing;

use super::types::TonSignMessageData;

pub fn sign_personal(data: &[u8], private_key: &[u8]) -> Result<(Vec<u8>, Vec<u8>), SignerError> {
    let ton_data = TonSignMessageData::from_bytes(data)?;
    let payload = ton_data.get_payload()?;
    let digest = payload.hash();

    let private_key = Zeroizing::new(private_key.to_vec());
    let signing_key = signing_key_from_bytes(&private_key)?;
    let signature = signing_key.sign(digest.as_slice());
    let signature_bytes = signature.to_bytes().to_vec();
    let public_key_bytes = signing_key.verifying_key().to_bytes().to_vec();

    Ok((signature_bytes, public_key_bytes))
}

fn signing_key_from_bytes(private_key: &[u8]) -> Result<ed25519_dalek::SigningKey, SignerError> {
    let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key
        .try_into()
        .map_err(|_| SignerError::InvalidInput("Invalid Ed25519 private key length".to_string()))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signature, SigningKey, Verifier};

    #[test]
    fn test_sign_ton_personal() {
        let payload = serde_json::json!({"type": "text", "text": "Hello TON"});
        let ton_data = TonSignMessageData::new(payload, "example.com".to_string());
        let data = ton_data.to_bytes();

        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").expect("valid hex");

        let (signature, public_key) = sign_personal(&data, &private_key).expect("signing succeeds");

        assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");
        assert_eq!(public_key.len(), 32, "Ed25519 public key should be 32 bytes");

        let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.try_into().expect("32 byte secret key");
        let signing_key = SigningKey::from_bytes(&key_bytes);
        assert_eq!(public_key, signing_key.verifying_key().as_bytes(), "public key should match");

        let signature = Signature::from_bytes(signature.as_slice().try_into().expect("64 byte signature"));
        let digest = b"Hello TON";
        signing_key.verifying_key().verify(digest, &signature).expect("signature verifies");
    }
}
