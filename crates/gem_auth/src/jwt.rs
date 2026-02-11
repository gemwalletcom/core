use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

pub fn create_device_token(device_id: &str, secret: &str, expiry: Duration) -> Result<(String, u64), jsonwebtoken::errors::Error> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let expires_at = now + expiry.as_secs();
    let claims = JwtClaims {
        sub: device_id.to_string(),
        exp: expires_at,
        iat: now,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
    Ok((token, expires_at))
}

pub fn verify_device_token(token: &str, secret: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    let token_data = decode::<JwtClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::new(Algorithm::HS256))?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify() {
        let secret = "test_secret_key_12345";
        let device_id = "abc123";
        let (token, expires_at) = create_device_token(device_id, secret, Duration::from_secs(3600)).unwrap();
        let claims = verify_device_token(&token, secret).unwrap();

        assert_eq!(claims.sub, device_id);
        assert_eq!(claims.exp, expires_at);
        assert_eq!(claims.exp - claims.iat, 3600);
    }

    #[test]
    fn test_wrong_secret() {
        let (token, _) = create_device_token("device1", "secret1", Duration::from_secs(3600)).unwrap();
        assert!(verify_device_token(&token, "wrong_secret").is_err());
    }

    #[test]
    fn test_expired_token() {
        let secret = "test_secret";
        let claims = JwtClaims {
            sub: "device1".to_string(),
            exp: 1000,
            iat: 900,
        };
        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap();
        assert!(verify_device_token(&token, secret).is_err());
    }

    #[test]
    fn test_invalid_token() {
        assert!(verify_device_token("not.a.valid.token", "secret").is_err());
    }
}
