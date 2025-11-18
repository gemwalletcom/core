use gem_evm::domain::host;
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

        let metadata_domain = host(&metadata_url);
        let origin_domain = host(&origin);

        if metadata_domain == origin_domain {
            WalletConnectionVerificationStatus::Verified
        } else {
            WalletConnectionVerificationStatus::Invalid
        }
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
