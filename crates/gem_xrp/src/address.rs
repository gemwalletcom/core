use std::fmt;

use primitives::Address;
#[cfg(feature = "signer")]
use primitives::SignerError;

const CLASSIC_ACCOUNT_ID_LENGTH: usize = 20;
const CLASSIC_ADDRESS_LENGTH: usize = CLASSIC_ACCOUNT_ID_LENGTH + 1;
const CLASSIC_PREFIX: u8 = 0x00;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XrpAddress([u8; CLASSIC_ACCOUNT_ID_LENGTH]);

impl Address for XrpAddress {
    fn try_parse(value: &str) -> Option<Self> {
        let decoded = bs58::decode(value).with_alphabet(bs58::Alphabet::RIPPLE).with_check(Some(CLASSIC_PREFIX)).into_vec().ok()?;

        if decoded.len() != CLASSIC_ADDRESS_LENGTH {
            return None;
        }

        let account_id = decoded[1..].try_into().ok()?;
        Some(Self(account_id))
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn encode(&self) -> String {
        let mut raw = Vec::with_capacity(CLASSIC_ADDRESS_LENGTH);
        raw.push(CLASSIC_PREFIX);
        raw.extend_from_slice(&self.0);
        bs58::encode(raw).with_alphabet(bs58::Alphabet::RIPPLE).with_check().into_string()
    }
}

impl XrpAddress {
    #[cfg(feature = "signer")]
    pub(crate) fn parse(value: &str) -> Result<Self, SignerError> {
        Self::try_parse(value).ok_or_else(|| SignerError::invalid_input("invalid XRP classic address"))
    }

    #[cfg(feature = "signer")]
    pub(crate) fn as_bytes(&self) -> &[u8; CLASSIC_ACCOUNT_ID_LENGTH] {
        &self.0
    }
}

pub fn validate_address(address: &str) -> bool {
    XrpAddress::is_valid(address)
}

impl fmt::Display for XrpAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLASSIC_ADDRESS: &str = "rnBFvgZphmN39GWzUJeUitaP22Fr9be75H";

    #[test]
    fn test_parse_addresses() {
        let parsed = XrpAddress::from_str(CLASSIC_ADDRESS).unwrap();

        assert_eq!(hex::encode(parsed.as_bytes()), "2decab42ca805119a9ba2ff305c9afa12f0b86a1");
        assert_eq!(parsed.to_string(), CLASSIC_ADDRESS);
        assert!(validate_address(CLASSIC_ADDRESS));
        assert!(XrpAddress::from_str("invalid").is_err());
        assert!(!validate_address("rnBFvgZphmN39GWzUJeUitaP22Fr9be75J"));
    }
}
