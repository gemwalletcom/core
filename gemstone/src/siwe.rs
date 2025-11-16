use alloy_primitives::hex;
use chrono::{DateTime, FixedOffset};
use primitives::{Chain, ChainType};
use std::collections::VecDeque;
use url::Url;

/// https://eips.ethereum.org/EIPS/eip-4361
const PREAMBLE_SUFFIX: &str = " wants you to sign in with your Ethereum account:";

#[derive(Debug, Clone, uniffi::Record)]
pub struct SiweMessage {
    pub domain: String,
    pub scheme: Option<String>,
    pub address: String,
    pub statement: Option<String>,
    pub uri: String,
    pub version: String,
    pub chain_id: u64,
    pub nonce: String,
    pub issued_at: String,
    pub expiration_time: Option<String>,
    pub not_before: Option<String>,
    pub request_id: Option<String>,
    pub resources: Vec<String>,
}

impl SiweMessage {
    pub fn try_parse(raw: &str) -> Option<Self> {
        let mut lines: VecDeque<&str> = raw.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let preamble = lines.pop_front()?.trim();
        if !preamble.ends_with(PREAMBLE_SUFFIX) {
            return None;
        }

        let domain = preamble[..preamble.len() - PREAMBLE_SUFFIX.len()].trim();
        if domain.is_empty() {
            return None;
        }

        let scheme = Self::parse_scheme(&mut lines)?;

        let address_line = lines.pop_front()?.trim().to_string();
        Self::validate_address(&address_line).ok()?;

        Self::consume_blank_lines(&mut lines);
        let statement = Self::parse_statement(&mut lines);

        let uri = Self::parse_required_tag(&mut lines, "URI: ", "URI")?;
        let version = Self::parse_required_tag(&mut lines, "Version: ", "Version")?;
        if version.trim() != "1" {
            return None;
        }

        let chain_id_text = Self::parse_required_tag(&mut lines, "Chain ID: ", "Chain ID")?;
        let chain_id = chain_id_text.parse().ok()?;

        let nonce = Self::parse_required_tag(&mut lines, "Nonce: ", "Nonce")?;
        let issued_at = Self::parse_required_tag(&mut lines, "Issued At: ", "Issued At")?;
        let expiration_time = Self::parse_optional_tag(&mut lines, "Expiration Time: ");
        let not_before = Self::parse_optional_tag(&mut lines, "Not Before: ");
        let request_id = Self::parse_optional_tag(&mut lines, "Request ID: ");
        let resources = Self::parse_resources(&mut lines);

        Some(Self {
            domain: domain.to_string(),
            scheme,
            address: address_line,
            statement,
            uri,
            version: version.trim().to_string(),
            chain_id,
            nonce,
            issued_at,
            expiration_time,
            not_before,
            request_id,
            resources,
        })
    }

    pub fn validate(&self, chain: Chain) -> Result<(), String> {
        if chain.chain_type() != ChainType::Ethereum {
            return Err(format!("Unsupported chain for SIWE: {chain}"));
        }

        Self::verify_chain_id(chain, self.chain_id)?;
        Self::verify_nonce(&self.nonce)?;
        Self::validate_statement(&self.statement)?;
        let parsed_uri = Self::validate_uri(&self.uri)?;
        Self::validate_datetime(&self.issued_at).map_err(|e| format!("Invalid Issued At value: {e}"))?;

        if let Some(value) = &self.expiration_time {
            Self::validate_datetime(value).map_err(|e| format!("Invalid Expiration Time value: {e}"))?;
        }

        if let Some(value) = &self.not_before {
            Self::validate_datetime(value).map_err(|e| format!("Invalid Not Before value: {e}"))?;
        }

        Self::validate_resources(&self.resources)?;
        Self::validate_domain(&self.domain, self.scheme.as_deref())?;
        Self::verify_origin(&self.domain, self.scheme.as_deref(), &parsed_uri)?;

        Ok(())
    }

    fn parse_scheme(lines: &mut VecDeque<&str>) -> Option<Option<String>> {
        Self::consume_blank_lines(lines);
        let Some(line) = lines.front().copied() else {
            return Some(None);
        };
        let trimmed = line.trim();
        if !trimmed.starts_with("Scheme: ") {
            return Some(None);
        }
        lines.pop_front();
        let value = trimmed.trim_start_matches("Scheme: ").trim().to_string();
        if value.is_empty() {
            return Some(None);
        }
        if Self::validate_scheme(&value).is_ok() { Some(Some(value)) } else { None }
    }

    fn parse_statement(lines: &mut VecDeque<&str>) -> Option<String> {
        Self::consume_blank_lines(lines);
        let line = lines.front().copied()?.trim();
        if line.is_empty() || line.starts_with("URI: ") || line.starts_with("Scheme: ") {
            return None;
        }

        let statement = lines.pop_front()?.trim().to_string();
        Self::consume_blank_lines(lines);
        if statement.is_empty() { None } else { Some(statement) }
    }

    fn parse_required_tag(lines: &mut VecDeque<&str>, tag: &str, _label: &str) -> Option<String> {
        Self::consume_blank_lines(lines);
        let line = lines.pop_front()?.trim().to_string();
        let value = line.strip_prefix(tag)?.trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }

    fn parse_optional_tag(lines: &mut VecDeque<&str>, tag: &str) -> Option<String> {
        Self::consume_blank_lines(lines);
        let line = lines.front()?.trim();
        if !line.starts_with(tag) {
            return None;
        }

        let value = line[tag.len()..].trim().to_string();
        lines.pop_front();
        if value.is_empty() { None } else { Some(value) }
    }

    fn parse_resources(lines: &mut VecDeque<&str>) -> Vec<String> {
        Self::consume_blank_lines(lines);
        let mut resources = Vec::new();
        if !matches!(lines.front(), Some(line) if line.trim() == "Resources:") {
            return resources;
        }

        lines.pop_front();
        while let Some(line) = lines.front() {
            let trimmed = line.trim();
            if !trimmed.starts_with("- ") {
                break;
            }
            let value = trimmed.trim_start_matches("- ").trim();
            if !value.is_empty() {
                resources.push(value.to_string());
            }
            lines.pop_front();
        }
        resources
    }

    fn validate_scheme(value: &str) -> Result<(), String> {
        let mut chars = value.chars();
        let first = chars.next().ok_or_else(|| "Scheme cannot be empty".to_string())?;
        if !first.is_ascii_alphabetic() {
            return Err("Scheme must start with an ASCII letter".to_string());
        }
        for ch in chars {
            if !(ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.')) {
                return Err("Scheme contains invalid characters".to_string());
            }
        }
        Ok(())
    }

    fn validate_statement(statement: &Option<String>) -> Result<(), String> {
        if let Some(value) = statement {
            if value.contains('\n') {
                return Err("SIWE statement must not include new lines".to_string());
            }
            if !value.is_ascii() {
                return Err("SIWE statement must be ASCII".to_string());
            }
        }
        Ok(())
    }

    fn validate_domain(domain: &str, scheme: Option<&str>) -> Result<(), String> {
        Self::extract_domain(domain, scheme).ok_or_else(|| "Invalid SIWE domain".to_string())?;
        Ok(())
    }

    fn validate_uri(uri: &str) -> Result<Url, String> {
        Url::parse(uri).map_err(|e| format!("Invalid SIWE URI: {e}"))
    }

    fn validate_datetime(value: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
        DateTime::parse_from_rfc3339(value)
    }

    fn validate_resources(resources: &[String]) -> Result<(), String> {
        for resource in resources {
            Url::parse(resource).map_err(|e| format!("Invalid SIWE resource: {e}"))?;
        }
        Ok(())
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

    fn verify_origin(domain: &str, scheme: Option<&str>, parsed_uri: &Url) -> Result<(), String> {
        let (domain_host, domain_port) = Self::extract_domain(domain, scheme).ok_or_else(|| "SIWE domain missing host information".to_string())?;

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

    fn extract_domain(domain: &str, scheme: Option<&str>) -> Option<(String, Option<u16>)> {
        let scheme = scheme.unwrap_or("https");
        let formatted = format!("{scheme}://{domain}");
        let parsed = Url::parse(&formatted).ok()?;
        Some((parsed.host_str()?.to_lowercase(), parsed.port_or_known_default()))
    }

    fn validate_address(address: &str) -> Result<(), String> {
        let normalized = address.trim();
        if !normalized.starts_with("0x") || normalized.len() != 42 {
            return Err("Invalid SIWE address".to_string());
        }
        hex::decode(&normalized[2..]).map_err(|_| "Invalid SIWE address".to_string())?;
        Ok(())
    }

    fn consume_blank_lines(lines: &mut VecDeque<&str>) {
        while matches!(lines.front(), Some(line) if line.trim().is_empty()) {
            lines.pop_front();
        }
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
            "Scheme: https",
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
        assert_eq!(siwe.scheme.as_deref(), Some("https"));
        assert_eq!(siwe.address, "0x6dD7802E6d44bE89a789C4bD60bD511B68F41c7c");
        assert_eq!(siwe.uri, "https://login.xyz");
        assert_eq!(siwe.chain_id, 1);
        assert_eq!(siwe.nonce, "8hK9pX32");
        assert_eq!(siwe.resources.len(), 2);
        assert!(siwe.validate(Chain::Ethereum).is_ok());
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
