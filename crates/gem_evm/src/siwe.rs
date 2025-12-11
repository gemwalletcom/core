use alloy_primitives::{Address, hex};
use chrono::{DateTime, Utc};
use gem_hash::keccak::keccak256;
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use primitives::{Chain, ChainType};
use url::Url;
use uuid::Uuid;

use crate::domain::{extract_host, parse_url};

const PREAMBLE_SUFFIX: &str = " wants you to sign in with your Ethereum account:";
const URI_PREFIX: &str = "URI:";
const VERSION_PREFIX: &str = "Version:";
const CHAIN_ID_PREFIX: &str = "Chain ID:";
const NONCE_PREFIX: &str = "Nonce:";
const ISSUED_AT_PREFIX: &str = "Issued At:";
const SUPPORTED_VERSION: &str = "1";
const MIN_NONCE_LENGTH: usize = 8;

pub fn create_message(domain: &str, uri: &str, address: &str, chain_id: u64, statement: &str) -> String {
    let nonce = Uuid::new_v4().to_string().replace("-", "");
    let issued_at = Utc::now().to_rfc3339();

    format!(
        "{} wants you to sign in with your Ethereum account:\n\
        {}\n\
        \n\
        {}\n\
        \n\
        URI: {}\n\
        Version: {}\n\
        Chain ID: {}\n\
        Nonce: {}\n\
        Issued At: {}",
        domain, address, statement, uri, SUPPORTED_VERSION, chain_id, nonce, issued_at
    )
}

pub fn verify_signature(message: &str, signature_hex: &str, expected_address: &str) -> bool {
    let Some(recovered) = recover_address(message, signature_hex) else {
        return false;
    };
    recovered.eq_ignore_ascii_case(expected_address)
}

fn eip191_hash(message: &str) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let prefixed = [prefix.as_bytes(), message.as_bytes()].concat();
    keccak256(&prefixed)
}

fn recover_address(message: &str, signature_hex: &str) -> Option<String> {
    let message_hash = eip191_hash(message);
    let signature_bytes = hex::decode(signature_hex.strip_prefix("0x").unwrap_or(signature_hex)).ok()?;

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
    let recovered_key = VerifyingKey::recover_from_prehash(&message_hash, &signature, recovery_id).ok()?;

    let public_key_bytes = recovered_key.to_encoded_point(false);
    let public_key_hash = keccak256(&public_key_bytes.as_bytes()[1..]);
    let address = format!("0x{}", hex::encode(&public_key_hash[12..]));

    Some(address)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SiweMessage {
    pub domain: String,
    pub address: String,
    pub uri: String,
    pub chain_id: u64,
    pub nonce: String,
    pub version: String,
    pub issued_at: String,
}

impl SiweMessage {
    pub fn try_parse(raw: &str) -> Option<Self> {
        let lines: Vec<_> = raw.lines().collect();

        let domain = lines.first()?.trim().strip_suffix(PREAMBLE_SUFFIX)?.trim();
        let domain = extract_host(domain)?;

        let address = lines.get(1)?.trim();
        address.parse::<Address>().ok()?;

        let body: Vec<_> = lines.iter().skip(2).map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

        let uri = Self::find_field(&body, URI_PREFIX)?;
        let version = Self::find_field(&body, VERSION_PREFIX)?;
        let chain_id = Self::find_field(&body, CHAIN_ID_PREFIX)?.parse().ok()?;
        let nonce = Self::find_field(&body, NONCE_PREFIX)?;
        let issued_at = Self::find_field(&body, ISSUED_AT_PREFIX)?;

        Some(Self {
            domain,
            address: address.to_string(),
            uri,
            chain_id,
            nonce,
            version,
            issued_at,
        })
    }

    pub fn validate(&self, chain: Chain) -> Result<(), String> {
        if chain.chain_type() != ChainType::Ethereum {
            return Err("Unsupported chain for SIWE".to_string());
        }

        let expected_chain_id = chain.network_id().parse::<u64>().map_err(|_| "Invalid chain".to_string())?;
        if expected_chain_id != self.chain_id {
            return Err("Chain ID mismatch".to_string());
        }

        if self.version != SUPPORTED_VERSION {
            return Err("Unsupported version".to_string());
        }

        if self.nonce.len() < MIN_NONCE_LENGTH || !self.nonce.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err("Invalid nonce".to_string());
        }

        DateTime::parse_from_rfc3339(&self.issued_at).map_err(|_| "Invalid timestamp".to_string())?;

        let uri = Url::parse(&self.uri).map_err(|_| "Invalid URI".to_string())?;
        let domain_url = parse_url(&self.domain).ok_or("Invalid domain".to_string())?;

        let uri_host = uri.host_str().ok_or("Invalid URI host".to_string())?;
        let domain_host = domain_url.host_str().ok_or("Invalid domain host".to_string())?;

        if !uri_host.eq_ignore_ascii_case(domain_host) {
            return Err("Origin mismatch".to_string());
        }

        if uri.port_or_known_default() != domain_url.port_or_known_default() {
            return Err("Origin mismatch".to_string());
        }

        Ok(())
    }

    fn find_field(lines: &[&str], prefix: &str) -> Option<String> {
        lines.iter().find(|line| line.starts_with(prefix)).and_then(|line| {
            let value = line.strip_prefix(prefix)?.trim();
            if value.is_empty() {
                return None;
            }
            Some(value.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_message() -> String {
        [
            "login.xyz wants you to sign in with your Ethereum account:",
            "0x6dD7802E6d44bE89a789C4bD60bD511B68F41c7c",
            "",
            "Sign in with Ethereum to the app.",
            "",
            "URI: https://login.xyz",
            "Version: 1",
            "Chain ID: 1",
            "Nonce: 8hK9pX32",
            "Issued At: 2024-04-01T12:00:00Z",
            "Expiration Time: 2024-04-02T12:00:00Z",
            "Not Before: 2024-04-01T11:00:00Z",
            "Request ID: abc-123",
            "Resources:",
            "- https://example.com/terms",
            "- https://example.com/privacy",
        ]
        .join("\n")
    }

    #[test]
    fn parses_valid_message() {
        let message = sample_message();
        let result = SiweMessage::try_parse(&message);
        assert!(result.is_some());
        let siwe = result.unwrap();
        assert_eq!(siwe.domain, "login.xyz");
        assert_eq!(siwe.address, "0x6dD7802E6d44bE89a789C4bD60bD511B68F41c7c");
        assert_eq!(siwe.uri, "https://login.xyz");
        assert_eq!(siwe.chain_id, 1);
        assert_eq!(siwe.nonce, "8hK9pX32");
        assert_eq!(siwe.version, "1");
        assert_eq!(siwe.issued_at, "2024-04-01T12:00:00Z");
        assert!(siwe.validate(Chain::Ethereum).is_ok());
    }

    #[test]
    fn parses_message_with_explicit_scheme() {
        let message = sample_message().replacen(
            "login.xyz wants you to sign in with your Ethereum account:",
            "https://login.xyz wants you to sign in with your Ethereum account:",
            1,
        );
        let siwe = SiweMessage::try_parse(&message).unwrap();
        assert_eq!(siwe.domain, "login.xyz");
    }

    #[test]
    fn parses_message_with_port() {
        let message = sample_message().replacen(
            "login.xyz wants you to sign in with your Ethereum account:",
            "login.xyz:8080 wants you to sign in with your Ethereum account:",
            1,
        );
        let siwe = SiweMessage::try_parse(&message).unwrap();
        assert_eq!(siwe.domain, "login.xyz:8080");
    }

    #[test]
    fn ignores_non_siwe_messages() {
        let raw = "hello world";
        let result = SiweMessage::try_parse(raw);
        assert!(result.is_none());
    }

    #[test]
    fn errors_on_chain_mismatch() {
        let message = sample_message();
        let siwe = SiweMessage::try_parse(&message).unwrap();
        let err = siwe.validate(Chain::Polygon).unwrap_err();
        assert!(err.contains("mismatch"));
    }

    #[test]
    fn errors_on_origin_mismatch() {
        let message = sample_message();
        let tampered = message.replace("https://login.xyz", "https://malicious.xyz");
        let siwe = SiweMessage::try_parse(&tampered).unwrap();
        let err = siwe.validate(Chain::Ethereum).unwrap_err();
        assert!(err.contains("mismatch"));
    }
}
