use base64::{Engine as _, engine::general_purpose};
use rsa::RsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::SigningKey;
use rsa::sha2::Sha512;
use rsa::signature::{RandomizedSigner, SignatureEncoding};

pub fn generate_rsa_pss_signature(private_key_base_64: &str, message: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let decoded = general_purpose::STANDARD.decode(private_key_base_64)?;
    let private_key = if let Ok(pem_str) = String::from_utf8(decoded.clone()) {
        RsaPrivateKey::from_pkcs8_pem(&pem_str)?
    } else {
        RsaPrivateKey::from_pkcs8_der(&decoded)?
    };
    let signing_key = SigningKey::<Sha512>::new(private_key);
    let mut rng = rsa::rand_core::OsRng;
    let signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());
    Ok(general_purpose::STANDARD.encode(signature.to_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rsa_pss_signature_invalid_key() {
        let message = r#"{"test":"data"}"#;
        let result = generate_rsa_pss_signature("invalid_base64", message);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_rsa_pss_signature_paybis_example() {
        let base64_private_key = "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";

        let request_body = r#"{
  "partnerUserId": "1bc166bd-1808-4009-8454-aceb47ba8753",
  "cryptoWalletAddress": null,
  "email": "doe.john@paybis.com",
  "applicantSumsubToken": "token_hash",
  "locale": "en"
}"#;

        let signature = generate_rsa_pss_signature(base64_private_key, request_body).expect("Failed to generate signature");

        assert!(!signature.is_empty());
    }
}
