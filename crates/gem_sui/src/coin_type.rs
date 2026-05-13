use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL};
use primitives::hex::decode_hex;

const SUI_ADDRESS_LENGTH: usize = 32;

pub fn full_coin_type(coin_type: &str) -> String {
    let Some((prefix, rest)) = coin_type.split_once("::") else {
        return coin_type.to_string();
    };
    match decode_hex(prefix) {
        Ok(bytes) if bytes.len() <= SUI_ADDRESS_LENGTH => {
            let mut padded = [0u8; SUI_ADDRESS_LENGTH];
            padded[SUI_ADDRESS_LENGTH - bytes.len()..].copy_from_slice(&bytes);
            format!("0x{}::{rest}", hex::encode(padded))
        }
        _ => coin_type.to_string(),
    }
}

pub fn coin_type_matches(a: &str, b: &str) -> bool {
    full_coin_type(a) == full_coin_type(b)
}

pub fn is_sui_coin(coin_type: &str) -> bool {
    coin_type == SUI_COIN_TYPE || coin_type == SUI_COIN_TYPE_FULL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_coin_type() {
        assert_eq!(
            full_coin_type("0x2::sui::SUI"),
            "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"
        );
        assert_eq!(
            full_coin_type("2::sui::SUI"),
            "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"
        );
        assert_eq!(
            full_coin_type("0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"),
            "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"
        );
        assert_eq!(full_coin_type("0xabc"), "0xabc");
        assert_eq!(full_coin_type("not-a-type::coin::COIN"), "not-a-type::coin::COIN");
    }

    #[test]
    fn test_coin_type_matches() {
        assert!(coin_type_matches("0x2::sui::SUI", "0x2::sui::SUI"));
        assert!(coin_type_matches("0x2::sui::SUI", "2::sui::SUI"));
        assert!(coin_type_matches("2::sui::SUI", "0x2::sui::SUI"));
        assert!(coin_type_matches(
            "0x2::sui::SUI",
            "0x0000000000000000000000000000000000000000000000000000000000000002::sui::SUI"
        ));
        assert!(!coin_type_matches("0x2::sui::SUI", "0x3::token::TOKEN"));
    }

    #[test]
    fn test_is_sui_coin() {
        assert!(is_sui_coin(SUI_COIN_TYPE));
        assert!(is_sui_coin(SUI_COIN_TYPE_FULL));
        assert!(!is_sui_coin("0x3::token::TOKEN"));
    }
}
