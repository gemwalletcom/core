use primitives::WalletConnectionVerificationStatus;

#[uniffi::remote(Enum)]
pub enum WalletConnectionVerificationStatus {
    Verified,
    Unknown,
    Invalid,
    Malicious,
}

pub struct WalletConnectVerifier;

impl WalletConnectVerifier {
    pub fn validate_origin(metadata_url: String, origin: Option<String>, validation: WalletConnectionVerificationStatus) -> WalletConnectionVerificationStatus {
        match validation {
            WalletConnectionVerificationStatus::Verified => Self::validate_verified_origin(metadata_url, origin),
            WalletConnectionVerificationStatus::Malicious => WalletConnectionVerificationStatus::Malicious,
            WalletConnectionVerificationStatus::Invalid => WalletConnectionVerificationStatus::Invalid,
            WalletConnectionVerificationStatus::Unknown => WalletConnectionVerificationStatus::Unknown,
        }
    }

    fn validate_verified_origin(metadata_url: String, verified_origin: Option<String>) -> WalletConnectionVerificationStatus {
        let Some(origin) = verified_origin else {
            return WalletConnectionVerificationStatus::Invalid;
        };

        let metadata_domain = Self::extract_domain(&metadata_url);
        let origin_domain = Self::extract_domain(&origin);

        if Self::matches_domain(&metadata_domain, &origin_domain) {
            WalletConnectionVerificationStatus::Verified
        } else {
            WalletConnectionVerificationStatus::Invalid
        }
    }

    fn extract_domain(url_string: &str) -> String {
        url::Url::parse(url_string)
            .ok()
            .and_then(|url| url.host_str().map(|h| h.to_lowercase()))
            .unwrap_or_else(|| url_string.to_lowercase())
    }

    fn matches_domain(domain1: &str, domain2: &str) -> bool {
        domain1.to_lowercase() == domain2.to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_validation() {
        let result = WalletConnectVerifier::validate_origin("https://app.uniswap.org".to_string(), None, WalletConnectionVerificationStatus::Unknown);
        assert!(matches!(result, WalletConnectionVerificationStatus::Unknown));
    }

    #[test]
    fn test_malicious_validation() {
        let result = WalletConnectVerifier::validate_origin(
            "https://app.uniswap.org".to_string(),
            Some("https://malicious.com".to_string()),
            WalletConnectionVerificationStatus::Malicious,
        );
        assert!(matches!(result, WalletConnectionVerificationStatus::Malicious));
    }

    #[test]
    fn test_verified_matching_origin() {
        let result = WalletConnectVerifier::validate_origin(
            "https://app.uniswap.org".to_string(),
            Some("https://app.uniswap.org".to_string()),
            WalletConnectionVerificationStatus::Verified,
        );
        assert!(matches!(result, WalletConnectionVerificationStatus::Verified));
    }

    #[test]
    fn test_verified_mismatched_origin() {
        let result = WalletConnectVerifier::validate_origin(
            "https://app.uniswap.org".to_string(),
            Some("https://different.com".to_string()),
            WalletConnectionVerificationStatus::Verified,
        );
        assert!(matches!(result, WalletConnectionVerificationStatus::Invalid));
    }

    #[test]
    fn test_invalid_validation() {
        let result = WalletConnectVerifier::validate_origin(
            "https://app.uniswap.org".to_string(),
            Some("https://app.uniswap.org".to_string()),
            WalletConnectionVerificationStatus::Invalid,
        );
        assert!(matches!(result, WalletConnectionVerificationStatus::Invalid));
    }
}
