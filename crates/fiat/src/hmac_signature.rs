use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn generate_hmac_from_bytes(key_bytes: &[u8], message: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key_bytes).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let signature = result.into_bytes();
    general_purpose::STANDARD.encode(signature)
}

pub fn generate_hmac_signature(secret_key: &str, message: &str) -> String {
    generate_hmac_from_bytes(secret_key.as_bytes(), message)
}

pub fn generate_hmac_signature_from_base64_key(base64_secret_key: &str, message: &str) -> Option<String> {
    let decoded_key = general_purpose::STANDARD.decode(base64_secret_key).ok()?;
    Some(generate_hmac_from_bytes(&decoded_key, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hmac_signature() {
        let secret = "test_secret";
        let message = "test_message";
        let signature = generate_hmac_signature(secret, message);
        assert_eq!(signature, "ZaIJF7XWibQHwbbgx6qd5AIh78SB/+WPJIXFHYIqzs4=");
    }

    #[test]
    fn test_generate_hmac_signature_from_base64_key() {
        let base64_secret = "dGVzdF9zZWNyZXRfa2V5"; // "test_secret_key" in base64
        let message = "?currencyCodeFrom=USD&currencyCodeTo=BTC&flow=buyCrypto&partnerId=test_api_key&requestedAmount=100&requestedAmountType=from&walletAddress=bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
        let signature = generate_hmac_signature_from_base64_key(base64_secret, message).unwrap();

        assert!(!signature.is_empty());
        assert!(signature.contains("=") || signature.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/'));
    }

    #[test]
    fn test_generate_hmac_signature_from_base64_key_invalid_base64() {
        let invalid_base64_secret = "not_valid_base64!!!";
        let message = "test_message";
        let signature = generate_hmac_signature_from_base64_key(invalid_base64_secret, message);

        // Should return None for invalid base64
        assert!(signature.is_none());
    }
}
