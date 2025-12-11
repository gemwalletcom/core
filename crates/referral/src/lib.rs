pub const SIWE_DOMAIN: &str = "gemwallet.com";
pub const SIWE_URI: &str = "https://gemwallet.com";
pub const SIWE_STATEMENT: &str = "Gem Wallet Authentication";

pub fn create_siwe_message(address: &str, chain_id: u64) -> String {
    gem_evm::siwe::create_message(SIWE_DOMAIN, SIWE_URI, address, chain_id, SIWE_STATEMENT)
}

pub fn verify_siwe_signature(message: &str, signature_hex: &str, expected_address: &str) -> bool {
    let Some(parsed) = gem_evm::siwe::SiweMessage::try_parse(message) else {
        return false;
    };

    if !parsed.address.eq_ignore_ascii_case(expected_address) {
        return false;
    }

    if parsed.domain != SIWE_DOMAIN {
        return false;
    }

    gem_evm::siwe::verify_signature(message, signature_hex, expected_address)
}

pub fn parse_siwe_message(message: &str) -> Option<gem_evm::siwe::SiweMessage> {
    gem_evm::siwe::SiweMessage::try_parse(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_siwe_message() {
        let message = create_siwe_message("0x1234567890123456789012345678901234567890", 1);
        assert!(message.contains("gemwallet.com wants you to sign in with your Ethereum account:"));
        assert!(message.contains("0x1234567890123456789012345678901234567890"));
        assert!(message.contains("Chain ID: 1"));
    }

    #[test]
    fn test_parse_siwe_message() {
        let message = create_siwe_message("0x1234567890123456789012345678901234567890", 1);
        let parsed = parse_siwe_message(&message);
        assert!(parsed.is_some());
        let p = parsed.unwrap();
        assert_eq!(p.address, "0x1234567890123456789012345678901234567890");
        assert_eq!(p.chain_id, 1);
        assert_eq!(p.domain, SIWE_DOMAIN);
    }
}
