use alloy_primitives::hex;
use ed25519_dalek::{Signature, VerifyingKey};

pub fn verify_device_signature(public_key_hex: &str, message: &str, signature_hex: &str) -> bool {
    let Ok(pk_bytes) = hex::decode(public_key_hex) else {
        return false;
    };
    let Ok(pk_array): Result<[u8; 32], _> = pk_bytes.try_into() else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pk_array) else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(signature_hex) else {
        return false;
    };
    let Ok(sig_array): Result<[u8; 64], _> = sig_bytes.try_into() else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_array);
    verifying_key.verify_strict(message.as_bytes(), &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn test_verify_valid_signature() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "1706000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        assert!(verify_device_signature(&public_key_hex, message, &signature_hex));
    }

    #[test]
    fn test_reject_invalid_signature() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "1706000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let wrong_sig = hex::encode([0u8; 64]);

        assert!(!verify_device_signature(&public_key_hex, message, &wrong_sig));
    }

    #[test]
    fn test_reject_tampered_message() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "1706000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let tampered = "1706000000.POST./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(!verify_device_signature(&public_key_hex, tampered, &signature_hex));
    }

    #[test]
    fn test_reject_invalid_hex() {
        assert!(!verify_device_signature("not_hex", "msg", "not_hex"));
        assert!(!verify_device_signature("aabb", "msg", "aabb"));
    }
}
