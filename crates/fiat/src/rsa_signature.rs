use base64::{Engine as _, engine::general_purpose};
use ring::signature::{RSA_PSS_SHA512, RsaKeyPair};
use sha2::{Digest, Sha512};

pub fn generate_rsa_pss_signature(base64_private_key: &str, message: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let sha512_hash = Sha512::digest(message.as_bytes());
    let hash_hex = hex::encode(sha512_hash).to_lowercase();

    let private_key_pem = String::from_utf8(general_purpose::STANDARD.decode(base64_private_key)?)?;

    let private_key_der = pem_to_der(&private_key_pem)?;

    let key_pair = RsaKeyPair::from_pkcs8(&private_key_der).map_err(|e| format!("Failed to parse RSA private key: {:?}", e))?;

    let rng = ring::rand::SystemRandom::new();
    let mut signature = vec![0u8; key_pair.public().modulus_len()];

    key_pair
        .sign(&RSA_PSS_SHA512, &rng, hash_hex.as_bytes(), &mut signature)
        .map_err(|e| format!("Failed to sign: {:?}", e))?;

    Ok(general_purpose::STANDARD.encode(&signature))
}

fn pem_to_der(pem: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let pem = pem.trim();
    let lines: Vec<&str> = pem.lines().collect();

    if lines.len() < 3 {
        return Err("Invalid PEM format".into());
    }

    let base64_content = lines[1..lines.len() - 1].join("");
    let der = general_purpose::STANDARD.decode(base64_content)?;

    Ok(der)
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
}
