use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn generate_hmac_signature(secret_key: &str, message: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    let signature = result.into_bytes();
    general_purpose::STANDARD.encode(signature)
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
}
