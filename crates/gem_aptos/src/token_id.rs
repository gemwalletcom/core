use primitives::decode_hex;

const FUNGIBLE_ASSET_TOKEN_ID_LENGTH: usize = 66;

pub(crate) fn is_fungible_asset_token_id(token_id: &str) -> bool {
    token_id.starts_with("0x")
        && token_id.len() >= FUNGIBLE_ASSET_TOKEN_ID_LENGTH
        && decode_hex(token_id).map(|bytes| bytes.len() == 32).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::asset_constants::APTOS_USDT_TOKEN_ID;

    #[test]
    fn test_is_fungible_asset_token_id() {
        assert!(is_fungible_asset_token_id(APTOS_USDT_TOKEN_ID));
        assert!(!is_fungible_asset_token_id("0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC"));
        assert!(!is_fungible_asset_token_id("0xa"));
        assert!(!is_fungible_asset_token_id("0xzz7b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b"));
        assert!(!is_fungible_asset_token_id("invalid"));
    }
}
