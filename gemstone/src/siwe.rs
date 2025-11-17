use alloy_primitives::hex;
use chrono::{DateTime, FixedOffset};
use primitives::{Chain, ChainType};
use url::Url;

/// https://eips.ethereum.org/EIPS/eip-4361
const PREAMBLE_SUFFIX: &str = " wants you to sign in with your Ethereum account:";

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
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
        let mut lines = raw.lines();
        let preamble = lines.next()?.trim();
        if !preamble.ends_with(PREAMBLE_SUFFIX) {
            return None;
        }

        let preamble_value = preamble[..preamble.len() - PREAMBLE_SUFFIX.len()].trim();
        let domain = Self::parse_domain(preamble_value)?;

        let address_line = lines.next()?.trim();
        Self::validate_address(address_line).ok()?;
        let address = address_line.to_string();

        let mut uri = None;
        let mut chain_id = None;
        let mut nonce = None;
        let mut version = None;
        let mut issued_at = None;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if uri.is_none()
                && let Some(value) = trimmed.strip_prefix("URI:")
            {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return None;
                }
                Url::parse(normalized).ok()?;
                uri = Some(normalized.to_string());
                continue;
            }

            if version.is_none()
                && let Some(value) = trimmed.strip_prefix("Version:")
            {
                let normalized = value.trim();
                if normalized.is_empty() || normalized != "1" {
                    return None;
                }
                version = Some(normalized.to_string());
                continue;
            }

            if chain_id.is_none()
                && let Some(value) = trimmed.strip_prefix("Chain ID:")
            {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return None;
                }
                chain_id = normalized.parse().ok();
                continue;
            }

            if nonce.is_none()
                && let Some(value) = trimmed.strip_prefix("Nonce:")
            {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return None;
                }
                nonce = Some(normalized.to_string());
                continue;
            }

            if issued_at.is_none()
                && let Some(value) = trimmed.strip_prefix("Issued At:")
            {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return None;
                }
                issued_at = Some(normalized.to_string());
                continue;
            }
        }

        Some(Self {
            domain,
            address,
            uri: uri?,
            chain_id: chain_id?,
            nonce: nonce?,
            version: version?,
            issued_at: issued_at?,
        })
    }

    pub fn validate(&self, chain: Chain) -> Result<(), String> {
        if chain.chain_type() != ChainType::Ethereum {
            return Err(format!("Unsupported chain for SIWE: {chain}"));
        }

        Self::verify_chain_id(chain, self.chain_id)?;
        Self::verify_nonce(&self.nonce)?;
        Self::validate_version(&self.version)?;
        Self::validate_datetime(&self.issued_at).map_err(|e| format!("Invalid Issued At value: {e}"))?;
        let parsed_uri = Self::validate_uri(&self.uri)?;
        Self::validate_domain(&self.domain)?;
        Self::verify_origin(&self.domain, &parsed_uri)?;

        Ok(())
    }

    fn parse_domain(value: &str) -> Option<String> {
        if value.is_empty() {
            return None;
        }

        if let Some(idx) = value.find("://") {
            let domain_part = value[idx + 3..].trim();
            if domain_part.is_empty() {
                return None;
            }
            Some(domain_part.to_string())
        } else {
            Some(value.to_string())
        }
    }

    fn validate_domain(domain: &str) -> Result<(), String> {
        Self::extract_domain(domain).ok_or_else(|| "Invalid SIWE domain".to_string())?;
        Ok(())
    }

    fn validate_uri(uri: &str) -> Result<Url, String> {
        Url::parse(uri).map_err(|e| format!("Invalid SIWE URI: {e}"))
    }

    fn validate_datetime(value: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(value)
    }

    fn verify_chain_id(chain: Chain, message_chain_id: u64) -> Result<(), String> {
        let expected_chain_id = chain.network_id().parse::<u64>().map_err(|_| format!("Unsupported chain for SIWE: {chain}"))?;

        if expected_chain_id == message_chain_id {
            Ok(())
        } else {
            Err(format!("SIWE chain ID mismatch: expected {expected_chain_id}, received {message_chain_id}"))
        }
    }

    fn verify_nonce(nonce: &str) -> Result<(), String> {
        if nonce.len() < 8 {
            return Err("SIWE nonce must be at least 8 characters".to_string());
        }
        if nonce.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(())
        } else {
            Err("SIWE nonce must be alphanumeric".to_string())
        }
    }

    fn verify_origin(domain: &str, parsed_uri: &Url) -> Result<(), String> {
        let (domain_host, domain_port) = Self::extract_domain(domain).ok_or_else(|| "SIWE domain missing host information".to_string())?;

        let uri_host = parsed_uri.host_str().ok_or_else(|| "SIWE URI missing host information".to_string())?;
        let uri_port = parsed_uri.port_or_known_default();

        if uri_host.eq_ignore_ascii_case(&domain_host) && uri_port == domain_port {
            Ok(())
        } else {
            Err(format!(
                "SIWE origin mismatch: domain {domain_host}:{:?} does not match URI {uri_host}:{:?}",
                domain_port, uri_port
            ))
        }
    }

    fn extract_domain(domain: &str) -> Option<(String, Option<u16>)> {
        let formatted = if domain.contains("://") {
            domain.to_string()
        } else {
            format!("https://{domain}")
        };
        let parsed = Url::parse(&formatted).ok()?;
        Some((parsed.host_str()?.to_lowercase(), parsed.port_or_known_default()))
    }

    fn validate_version(version: &str) -> Result<(), String> {
        if version.trim() == "1" {
            Ok(())
        } else {
            Err("Unsupported SIWE version".to_string())
        }
    }

    fn validate_address(address: &str) -> Result<(), String> {
        let normalized = address.trim();
        if !normalized.starts_with("0x") || normalized.len() != 42 {
            return Err("Invalid SIWE address".to_string());
        }
        hex::decode(&normalized[2..]).map_err(|_| "Invalid SIWE address".to_string())?;
        Ok(())
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
        assert!(err.contains("chain ID mismatch"));
    }

    #[test]
    fn errors_on_origin_mismatch() {
        let message = sample_message();
        let tampered = message.replace("https://login.xyz", "https://malicious.xyz");
        let siwe = SiweMessage::try_parse(&tampered).unwrap();
        let err = siwe.validate(Chain::Ethereum).unwrap_err();
        assert!(err.contains("origin mismatch"));
    }
}
