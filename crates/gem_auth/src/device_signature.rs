use alloy_primitives::hex;
use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, VerifyingKey};

pub const GEM_AUTH_SCHEME: &str = "Gem ";

#[derive(Debug, PartialEq)]
pub enum AuthScheme {
    Gem,
    Legacy,
}

pub struct DeviceAuthPayload {
    pub scheme: AuthScheme,
    pub device_id: String,
    pub timestamp: String,
    pub wallet_id: Option<String>,
    pub body_hash: String,
    pub signature: Vec<u8>,
}

pub fn parse_device_auth(header_value: &str) -> Option<DeviceAuthPayload> {
    let encoded = header_value.strip_prefix(GEM_AUTH_SCHEME)?;
    let decoded = STANDARD.decode(encoded).ok()?;
    let payload = String::from_utf8(decoded).ok()?;
    let parts: Vec<&str> = payload.splitn(5, '.').collect();
    if parts.len() != 5 {
        return None;
    }
    Some(DeviceAuthPayload {
        scheme: AuthScheme::Gem,
        device_id: parts[0].to_string(),
        timestamp: parts[1].to_string(),
        wallet_id: if parts[2].is_empty() { None } else { Some(parts[2].to_string()) },
        body_hash: parts[3].to_string(),
        signature: hex::decode(parts[4]).ok()?,
    })
}

// TODO: remove base64 fallback once all clients use hex signatures
pub fn decode_signature(value: &str) -> Option<Vec<u8>> {
    hex::decode(value).ok().or_else(|| STANDARD.decode(value).ok())
}

pub fn verify_device_signature(public_key_hex: &str, message: &str, signature: &[u8]) -> bool {
    let Ok(pk_bytes) = hex::decode(public_key_hex) else {
        return false;
    };
    let Ok(pk_array): Result<[u8; 32], _> = pk_bytes.try_into() else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pk_array) else {
        return false;
    };
    let Ok(sig_array): Result<[u8; 64], _> = signature.try_into() else {
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
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());

        assert!(verify_device_signature(&public_key_hex, message, &signature.to_bytes()));
    }

    #[test]
    fn test_reject_invalid_signature() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        assert!(!verify_device_signature(&public_key_hex, message, &[0u8; 64]));
    }

    #[test]
    fn test_reject_tampered_message() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());

        let tampered = "v1.1706000000000.POST./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(!verify_device_signature(&public_key_hex, tampered, &signature.to_bytes()));
    }

    #[test]
    fn test_reject_wrong_public_key() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let wrong_key = SigningKey::from_bytes(&[2u8; 32]);
        let wrong_public_key_hex = hex::encode(wrong_key.verifying_key().as_bytes());
        let message = "v1.1706000000000.GET./v1/devices/abc.e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(message.as_bytes());

        assert!(!verify_device_signature(&wrong_public_key_hex, message, &signature.to_bytes()));
    }

    #[test]
    fn test_reject_invalid_signature_length() {
        assert!(!verify_device_signature("aabb", "msg", &[0u8; 2]));
    }

    #[test]
    fn test_parse_device_auth() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let timestamp = "1706000000000";
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let wallet_id = "multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let signature = signing_key.sign(b"test");
        let signature_hex = hex::encode(signature.to_bytes());

        let payload = format!("{}.{}.{}.{}.{}", public_key_hex, timestamp, wallet_id, body_hash, signature_hex);
        let encoded = STANDARD.encode(payload.as_bytes());
        let header = format!("Gem {}", encoded);

        let result = parse_device_auth(&header).unwrap();
        assert_eq!(result.device_id, public_key_hex);
        assert_eq!(result.timestamp, timestamp);
        assert_eq!(result.wallet_id.as_deref(), Some(wallet_id));
        assert_eq!(result.body_hash, body_hash);
        assert_eq!(result.signature, signature.to_bytes());
    }

    #[test]
    fn test_parse_device_auth_invalid() {
        assert!(parse_device_auth("Bearer token").is_none());
        assert!(parse_device_auth("Gem !!!").is_none());
        let encoded = STANDARD.encode(b"only.two.parts");
        assert!(parse_device_auth(&format!("Gem {}", encoded)).is_none());
    }

    #[test]
    fn test_parse_device_auth_empty_wallet_id() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let timestamp = "1706000000000";
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let signature = signing_key.sign(b"test");
        let signature_hex = hex::encode(signature.to_bytes());

        let payload = format!("{}.{}..{}.{}", public_key_hex, timestamp, body_hash, signature_hex);
        let encoded = STANDARD.encode(payload.as_bytes());
        let header = format!("Gem {}", encoded);

        let result = parse_device_auth(&header).unwrap();
        assert_eq!(result.device_id, public_key_hex);
        assert_eq!(result.timestamp, timestamp);
        assert_eq!(result.wallet_id, None);
        assert_eq!(result.body_hash, body_hash);
    }

    #[test]
    fn test_verify_signature_with_wallet_id() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let wallet_id = "multicoin_0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let message = format!("1706000000000.GET./v1/devices/abc.{}.{}", wallet_id, body_hash);
        let signature = signing_key.sign(message.as_bytes());

        assert!(verify_device_signature(&public_key_hex, &message, &signature.to_bytes()));
    }

    #[test]
    fn test_verify_signature_empty_wallet_id() {
        let signing_key = SigningKey::from_bytes(&[1u8; 32]);
        let public_key_hex = hex::encode(signing_key.verifying_key().as_bytes());
        let body_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let message = format!("1706000000000.GET./v1/devices/abc..{}", body_hash);
        let signature = signing_key.sign(message.as_bytes());

        assert!(verify_device_signature(&public_key_hex, &message, &signature.to_bytes()));
    }
}
