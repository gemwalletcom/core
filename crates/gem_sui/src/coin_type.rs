use crate::{SUI_COIN_TYPE, SUI_COIN_TYPE_FULL};

pub fn full_coin_type(coin_type: &str) -> String {
    let Some(index) = coin_type.find("::") else {
        return coin_type.to_string();
    };
    let prefix = &coin_type[..index];
    let rest = &coin_type[index..];
    let Some(hex) = prefix.strip_prefix("0x") else {
        return coin_type.to_string();
    };
    if hex.len() > 64 {
        return coin_type.to_string();
    }
    format!("0x{hex:0>64}{rest}")
}

pub fn coin_type_matches(coin_type: &str, token_id: &str) -> bool {
    let coin_type = coin_type.strip_prefix("0x").unwrap_or(coin_type).to_lowercase();
    let token_id = token_id.strip_prefix("0x").unwrap_or(token_id).to_lowercase();

    coin_type == token_id
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
        assert!(!coin_type_matches("0x2::sui::SUI", "0x3::token::TOKEN"));
    }

    #[test]
    fn test_is_sui_coin() {
        assert!(is_sui_coin(SUI_COIN_TYPE));
        assert!(is_sui_coin(SUI_COIN_TYPE_FULL));
        assert!(!is_sui_coin("0x3::token::TOKEN"));
    }
}
